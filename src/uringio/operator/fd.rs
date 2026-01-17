use std::os::fd::AsRawFd;

use crate::platform::iouring::{AsFd, IoUringSqeFlags, NopFlags, RawFd};

pub trait OpFd {
    const SQE_FLAG: IoUringSqeFlags;

    const NOP_FLAG: u32;

    fn raw_fd(&self) -> RawFd;
}

/// FixFd
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct FixFd {
    idx: RawFd,
}

impl From<usize> for FixFd {
    /// Restrict: idx <= Fd limit <= i32::MAX
    #[inline]
    fn from(idx: usize) -> Self {
        // FIXME: overflow
        Self { idx: idx as _ }
    }
}

impl OpFd for FixFd {
    const NOP_FLAG: u32 = NopFlags::FILE | NopFlags::FIXED_FILE;
    const SQE_FLAG: IoUringSqeFlags = IoUringSqeFlags::FIXED_FILE;

    #[inline]
    fn raw_fd(&self) -> RawFd {
        self.idx
    }
}

impl<T> OpFd for T
where
    T: AsFd,
{
    const NOP_FLAG: u32 = NopFlags::FILE;
    const SQE_FLAG: IoUringSqeFlags = IoUringSqeFlags::empty();

    #[inline]
    fn raw_fd(&self) -> RawFd {
        self.as_fd().as_raw_fd()
    }
}
