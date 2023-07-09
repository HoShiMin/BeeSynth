pub trait PortAccessor {
    fn read_byte(&self, port_number: u16) -> Option<u8>;
    fn write_byte(&self, port_number: u16, value: u8) -> bool;
}