use windows::{
    Win32::{
        System::{
            Threading::{
                CreateEventW,
                SetEvent,
                ResetEvent,
                WaitForSingleObject,
                INFINITE, WaitForMultipleObjects,
            },
            WindowsProgramming::SignalObjectAndWait
        },
        Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT, WAIT_ABANDONED}
    },
    core::{ PCWSTR, Error }
};

use crate::auto;

#[derive(Default)]
pub struct Event {
    event_handle: auto::Handle
}

#[derive(Debug, Clone)]
pub enum WaitStatus {
    Signaled(u32), // Zero-relative number of a signaled object
    Abandoned,
    Timeout,
    Failure(Error)
}

impl WaitStatus {
    #[must_use]
    pub fn is_signaled(&self) -> bool {
        matches!(*self, Self::Signaled(_))
    }

    #[must_use]
    pub fn is_timeout(&self) -> bool {
        matches!(*self, Self::Timeout)
    }

    #[must_use]
    pub fn is_failure(&self) -> bool {
        matches!(*self, Self::Abandoned | Self::Failure(_))
    }
}

impl Event {
    /// # Panics
    /// 
    /// Will panic in case of `CreateEvent` failure
    #[must_use]
    pub fn new(manual_reset: bool, is_initially_signaled: bool) -> Self {
        let handle = unsafe {
            CreateEventW(
                None,
                manual_reset,
                is_initially_signaled,
                PCWSTR::null()
            ).unwrap()
        };
        Event { event_handle: auto::Handle::new(handle) }
    }

    #[must_use]
    pub fn new_manual(is_initially_signaled: bool) -> Self {
        Self::new(true, is_initially_signaled)
    }

    #[must_use]
    pub fn new_auto(is_initially_signaled: bool) -> Self {
        Self::new(false, is_initially_signaled)
    }

    pub fn set_event(&self) {
        unsafe { SetEvent(self.event_handle.get()) };
    }

    pub fn reset_event(&self) {
        unsafe { ResetEvent(self.event_handle.get()) };
    }

    #[must_use]
    pub fn wait(&self) -> WaitStatus {
        self.wait_timeout(INFINITE)
    }

    #[must_use]
    pub fn wait_timeout(&self, timeout_msec: u32) -> WaitStatus {
        let wait_status = unsafe { WaitForSingleObject(self.event_handle.get(), timeout_msec) };
        match wait_status {
            WAIT_OBJECT_0 => WaitStatus::Signaled(0),
            WAIT_TIMEOUT => WaitStatus::Timeout,
            WAIT_ABANDONED => WaitStatus::Abandoned,
            _ => WaitStatus::Failure(Error::from_win32())
        }
    }

    #[must_use]
    pub fn wait_many(objects: &[HANDLE], wait_all: bool, timeout_msec: u32) -> WaitStatus {
        let wait_status = unsafe { WaitForMultipleObjects(objects, wait_all, timeout_msec) };

        #[allow(clippy::cast_possible_truncation)]
        if wait_status.0 < WAIT_OBJECT_0.0 + objects.len() as u32 {
            WaitStatus::Signaled(wait_status.0 - WAIT_OBJECT_0.0)
        } else {
            match wait_status {
                WAIT_TIMEOUT => WaitStatus::Timeout,
                WAIT_ABANDONED => WaitStatus::Abandoned,
                _ => WaitStatus::Failure(Error::from_win32())
            }
        }
    }

    #[must_use]
    pub fn signal_and_wait_raw(object_to_signal: HANDLE, object_to_wait: HANDLE, timeout_msec: u32) -> WaitStatus {
        let wait_status = unsafe { SignalObjectAndWait(object_to_signal, object_to_wait, timeout_msec, false) };
        match wait_status {
            WAIT_OBJECT_0 => WaitStatus::Signaled(0),
            WAIT_TIMEOUT => WaitStatus::Timeout,
            WAIT_ABANDONED => WaitStatus::Abandoned,
            _ => WaitStatus::Failure(Error::from_win32())
        }
    }

    #[must_use]
    pub fn signal_and_wait(object_to_signal: &Event, object_to_wait: &Event, timeout_msec: u32) -> WaitStatus {
        Self::signal_and_wait_raw(object_to_signal.handle().get(), object_to_wait.handle().get(), timeout_msec)
    }

    #[must_use]
    pub fn handle(&self) -> &auto::Handle {
        &self.event_handle
    }

    #[must_use]
    pub fn handle_mut(&mut self) -> &mut auto::Handle {
        &mut self.event_handle
    }
}

unsafe impl Send for Event {}
unsafe impl Sync for Event {}