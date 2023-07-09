pub fn from<T: Sized>(data: &T) -> &[u8] {
    unsafe {
        std::slice::from_raw_parts(
            std::ptr::addr_of!(*data).cast(),
            std::mem::size_of::<T>(),
        )
    }
}

pub fn from_mut<T: Sized>(data: &mut T) -> &mut [u8] {
    unsafe {
        std::slice::from_raw_parts_mut(
            std::ptr::addr_of_mut!(*data).cast(),
            std::mem::size_of::<T>(),
        )
    }
}