pub trait Mapping {
    fn size(&self) -> usize;

    fn mapping(&self) -> &[u8];
    fn mapping_mut(&mut self) -> &mut [u8];

    fn unmap(self);
}

pub trait Mapper<'a>: Sync + Send {
    type Type: Mapping + Sized;
    fn map(&'a self, phys_addr: u64, size: u64) -> Option<Self::Type>;
}