use std::mem::ManuallyDrop;

use crate::{
    platform::iouring::{io_uring_enter, AsFd, BorrowedFd, IoUringEnterFlags, OwnedFd},
    shared::error::Result,
    uringio::mmap_arena::MmapArena,
};

#[derive(Debug)]
pub struct RingDesc<S, C, M> {
    fd: ManuallyDrop<OwnedFd>,
    pub arena: ManuallyDrop<MmapArena<S, C, M>>,
    // TODO: desc flags
    pub enter_flags: IoUringEnterFlags,
}

impl<S, C, M> RingDesc<S, C, M> {
    pub fn new(fd: OwnedFd, arena: MmapArena<S, C, M>) -> Self {
        Self {
            fd: ManuallyDrop::new(fd),
            arena: ManuallyDrop::new(arena),
            enter_flags: Default::default(),
        }
    }

    pub fn get_events(&self) -> Result<u32> {
        let flags = IoUringEnterFlags::GETEVENTS | self.enter_flags;
        Ok(unsafe { io_uring_enter(self, 0, 0, flags)? })
    }
}

impl<S, C, M> AsFd for RingDesc<S, C, M> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

impl<S, C, M> Drop for RingDesc<S, C, M> {
    fn drop(&mut self) {
        unsafe {
            // Mmap::drop munmap() preceding close(fd)
            ManuallyDrop::drop(&mut self.arena);

            if !self.enter_flags.contains(IoUringEnterFlags::REGISTERED_RING) {
                ManuallyDrop::drop(&mut self.fd);
            }
        }
    }
}
