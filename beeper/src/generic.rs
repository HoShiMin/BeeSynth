use crate::{
    sound_emitter::{
        BeeperDivisor,
        BeeperFrequency,
        SoundEmitter
    },
    port_accessor::PortAccessor
};

pub struct Beeper<'a, IoPorts: PortAccessor> {
    port_accessor: &'a IoPorts,
    control_port_value: u8
}

impl<'a, IoPorts: PortAccessor> Beeper<'a, IoPorts> {
    #[must_use]
    pub fn new(port_accessor: &'a IoPorts) -> Self {
        Self { port_accessor, control_port_value: 0 }
    }
}

impl<'a, IoPorts: PortAccessor> SoundEmitter for Beeper<'a, IoPorts> {
    #[must_use]
    fn prepare(&mut self) -> bool {
        // Set beeper regime:
        if !self.port_accessor.write_byte(0x43, 0xB6) {
            return false;
        }

        // Read the beeper control port value:
        self.control_port_value = if let Some(value) = self.port_accessor.read_byte(0x61) {
            value
        } else {
            return false;
        };

        true
    }

    fn play(&mut self) {
        let _write_status = self.port_accessor.write_byte(0x61, self.control_port_value | 0b11);
    }

    fn mute(&mut self) {
        let _write_status = self.port_accessor.write_byte(0x61, self.control_port_value & !0b11);
    }

    fn set_divisor(&mut self, divisor: BeeperDivisor) {
        #[allow(clippy::cast_possible_truncation)]
        let _write_status = self.port_accessor.write_byte(0x42, (divisor.get()) as u8);
        let _write_status = self.port_accessor.write_byte(0x42, (divisor.get() >> 8) as u8);
    }

    fn set_frequency(&mut self, freq: BeeperFrequency) {
        self.set_divisor(freq.into());
    }

    fn up(&mut self) {
        let _write_status = self.port_accessor.write_byte(0x61, self.control_port_value | 0b10);
    }

    fn down(&mut self) {
        let _write_status = self.port_accessor.write_byte(0x61, self.control_port_value & !0b10);
    }
}
