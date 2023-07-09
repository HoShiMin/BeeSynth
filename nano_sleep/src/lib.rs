#![warn(clippy::pedantic)]

use std::{
    arch::x86_64,
    time::{Instant, Duration}
};

pub trait NanoWaiter {
    fn nano_sleep(&self, nanoseconds: u64);
    fn ticks_in_nanosecond(&self) -> f32;
}


#[derive(Copy, Clone)]
pub struct NanoSleep {
    tsc_ticks_in_one_microsecond: u64,
    correction_tick_count: u64
}

impl NanoSleep {
    #[must_use]
    pub fn new(calibration_msec: u32) -> Self {

        const US_IN_MSEC: u64 = 1_000;

        let cpuid = raw_cpuid::CpuId::new();
        if let Some(apm_info) = cpuid.get_advanced_power_mgmt_info() {
            if apm_info.has_invariant_tsc() {
                if let Some(freq_info) = cpuid.get_processor_frequency_info() {
                    const CORRECTION_COUNT: usize = 30;
                    const CORRECTION_WAIT_NSEC: u64 = 800;
                    const WARMING_COUNT: usize = 10;

                    let base_tsc_freq_mhz = u64::from(freq_info.processor_base_frequency());
                    let mut waiter = Self { tsc_ticks_in_one_microsecond: base_tsc_freq_mhz, correction_tick_count: 0 };
                  
                    for _ in 0..WARMING_COUNT {
                        waiter.nano_sleep(CORRECTION_WAIT_NSEC);
                    }
                    
                    let mut deltas: [u64; CORRECTION_COUNT] = [0; CORRECTION_COUNT];
                    for delta in &mut deltas {
                        let t1 = unsafe { x86_64::_rdtsc() };
                        waiter.nano_sleep(CORRECTION_WAIT_NSEC);
                        let t2 = unsafe { x86_64::_rdtsc() };
                        *delta = t2 - t1;
                    }

                    //
                    // Perform filtering for possible outstanding values.
                    // For example, we have an array:
                    // [ 70, 73, 68, 3000, 71, 71, 69, 8000, 65, 78 ]
                    //               ----              ----
                    //       These are strange values that we should exclude.
                    //
                    // Step 1: Sort the array.
                    // [ 65, 68, 69, 70, 71, 71, 73, 78, 3000, 8000 ]
                    //                   ---^---
                    //       The median that divides the array into two parts.
                    //       We don't need the median value, but we do need
                    //       the median value for the left and right parts.
                    //
                    // Step 2: Calc medians for the left and right parts.
                    // [ 65, 68, 69, 70, 71 ] [ 71, 73, 78, 3000, 8000 ]
                    //           --                     --
                    //       Quartile 1             Quartile 3
                    //
                    // Step 3: Calc the median range.
                    // Range = Q3 - Q1
                    // Range = 78 - 69 = 9
                    //
                    // Step 4: Calc the major bounds.
                    // Bounds = [(Q1 - (Range * 3)), (Q3 + (Range * 3))]
                    // Bounds = [(69 - 27), (78 + 27)] = [42, 105]
                    //
                    // Step 5: Exclude all values that are out of bounds.
                    // 3000 and 5000 are out of bounds, so they will be dropped.
                    //

                    deltas.sort_unstable();

                    let calc_median = |sorted_array: &[u64]| -> u64 {
                        if sorted_array.len() % 2 == 0 {
                            let high_mid_index = sorted_array.len() / 2;
                            (sorted_array[high_mid_index - 1] + sorted_array[high_mid_index]) / 2
                        } else {
                            sorted_array[sorted_array.len() / 2]
                        }
                    };

                    let left_half = &deltas[0..(deltas.len() / 2)];
                    let right_half = &deltas[(if deltas.len() % 2 == 0 { deltas.len() / 2 } else { deltas.len() / 2 + 1 })..];
                    
                    let quartile_1 = calc_median(left_half);
                    let quartile_3 = calc_median(right_half);

                    let interquartile_range = quartile_3 - quartile_1;

                    let major_bounds = {
                        let shift = interquartile_range * 3;
                        (quartile_1 - shift, quartile_3 + shift)
                    };

                    let mut actual_count: usize = 0;
                    let mut skipped_count: usize = 0;
                    for i in 0..deltas.len() {
                        if (deltas[i] < major_bounds.0) || (deltas[i] > major_bounds.1) {
                            skipped_count += 1;
                            continue;
                        }
                        actual_count += 1;
                        deltas[i - skipped_count] = deltas[i];
                    }

                    let average = deltas[0..actual_count].iter().sum::<u64>() / actual_count as u64;
                    let measured_tick_count = CORRECTION_WAIT_NSEC * base_tsc_freq_mhz / 1000;
                    
                    let correction_tick_count = average - measured_tick_count;

                    waiter.correction_tick_count = correction_tick_count;
                    return waiter;
                }
            }
        }

        //
        // Fallback for CPUs with a non-invariant TSC.
        // We do not use correction here because time-based
        // measurements are too inaccurate (at least in 100 ns.)
        // and therefore correction does not make sense.
        //

        let time_begin = Instant::now();
        let time_end = time_begin + Duration::from_millis(u64::from(calibration_msec));
        let ticks_begin = unsafe { x86_64::_rdtsc() };
        while Instant::now() < time_end {}
        let ticks_end = unsafe { x86_64::_rdtsc() };

        let ticks_delta = ticks_end - ticks_begin;
        
        #[allow(clippy::cast_precision_loss)]
        let tsc_ticks_in_usec = ticks_delta / (u64::from(calibration_msec) * US_IN_MSEC);

        Self { tsc_ticks_in_one_microsecond: tsc_ticks_in_usec, correction_tick_count: 0 }
    }
}

impl NanoWaiter for NanoSleep {
    #[inline]
    fn nano_sleep(&self, nanoseconds: u64) {
        if nanoseconds < 8 {
            return;
        } else if nanoseconds <= 12 {
            unsafe { x86_64::_rdtsc() }; // Just consume 20-30 ticks (see Agner Fog's tables: https://www.agner.org/optimize/instruction_tables.pdf)
            return;
        }

        let current_ticks = unsafe { x86_64::_rdtsc() };
        
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
        let end_ticks = current_ticks - self.correction_tick_count + (nanoseconds * self.tsc_ticks_in_one_microsecond) / 1000;
        
        while unsafe { x86_64::_rdtsc() } < end_ticks {}
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    fn ticks_in_nanosecond(&self) -> f32 {
        (self.tsc_ticks_in_one_microsecond as f32) / 1000_f32
    }
}



#[test]
fn test() {
    #![allow(clippy::cast_precision_loss)]

    const ITER_COUNT: usize = 15;
    const WAIT_TIME_NS: u64 = 10;

    let mut values: [u64; ITER_COUNT] = [0; ITER_COUNT];

    let waiter = NanoSleep::new(300);

    for entry in &mut values {
        let t1 = unsafe { x86_64::_rdtsc() };
        waiter.nano_sleep(WAIT_TIME_NS);
        let t2 = unsafe { x86_64::_rdtsc() };
        *entry = t2 - t1;
    }

    for value in values {
        let elapsed_nsec = value as f32 / waiter.ticks_in_nanosecond();
        let diff = elapsed_nsec - WAIT_TIME_NS as f32;
        let percentage = (elapsed_nsec * 100_f32) / (WAIT_TIME_NS as f32) - 100_f32;
        println!("Elapsed: {elapsed_nsec:.1} ns., deviation: {diff:.1} ns. ({percentage:.1}%)");
    }
}