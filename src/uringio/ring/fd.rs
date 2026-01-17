use std::marker::PhantomData;

use crate::{
    platform::iouring::{io_uring_setup, AsFd, BorrowedFd, IoUringParams, OwnedFd},
    shared::error::Result,
};

/// RingFd
#[derive(Debug)]
pub struct RingFd<S, C, M> {
    pub fd: OwnedFd,
    pub params: IoUringParams, // TODO: clean unused params

    _marker_: PhantomData<(S, C, M)>,
}

impl<S, C, M> RingFd<S, C, M> {
    pub const fn new(fd: OwnedFd, params: IoUringParams) -> Self {
        Self { fd, params, _marker_: PhantomData }
    }

    pub fn setup(mut params: IoUringParams) -> Result<Self> {
        let fd = unsafe { io_uring_setup(params.sq_entries, &mut params)? };
        Ok(Self::new(fd, params))
    }
}

impl<S, C, M> AsFd for RingFd<S, C, M> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

/// TODO: FixRingFd index
pub struct FixRingFd<S, C, M> {
    pub idx: u32,
    pub params: IoUringParams,

    _marker_: PhantomData<(S, C, M)>,
}

impl<S, C, M> FixRingFd<S, C, M> {
    pub const fn new(idx: u32, params: IoUringParams) -> Self {
        Self { idx, params, _marker_: PhantomData }
    }
}
