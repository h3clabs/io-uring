pub mod mode;

use std::{io::Result, marker::PhantomData};

use crate::{
    platform::iouring::{io_uring_setup, AsFd, BorrowedFd, IoUringParams, OwnedFd},
    uringio::{
        completion::{
            entry::{CompletionEntry, Cqe16, Cqe32, CqeMix},
            queue::CompletionQueue,
        },
        mmap_arena::MmapArena,
        setup_args::SetupArgs,
        submission::{
            entry::{Sqe128, Sqe64, SqeMix, SubmissionEntry},
            queue::SubmissionQueue,
        },
        uring::mode::Mode,
    },
};

/// UringFd
#[derive(Debug)]
pub struct UringFd<S, C, M> {
    pub fd: OwnedFd,
    pub params: IoUringParams,

    _marker_: PhantomData<(S, C, M)>,
}

impl<S, C, M> UringFd<S, C, M> {
    pub const fn new(fd: OwnedFd, params: IoUringParams) -> Self {
        Self { fd, params, _marker_: PhantomData }
    }

    pub fn setup(args: SetupArgs<S, C, M>) -> Result<Self> {
        let SetupArgs { mut params, .. } = args;
        let fd = unsafe { io_uring_setup(params.sq_entries, &mut params)? };
        Ok(Self::new(fd, params))
    }
}

impl<S, C, M> AsFd for UringFd<S, C, M> {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}

/// Uring
#[derive(Debug)]
pub struct Uring<'fd, S, C, M> {
    pub sq: SubmissionQueue<'fd, S, M>,
    pub cq: CompletionQueue<'fd, C, M>,
    pub arena: MmapArena<'fd, S, C>,
}

impl<'fd, S, C, M> Uring<'fd, S, C, M>
where
    S: SubmissionEntry,
    C: CompletionEntry,
    M: Mode,
{
    pub fn new(fd: &'fd UringFd<S, C, M>) -> Result<Self> {
        unsafe {
            let arena = MmapArena::new(fd, &fd.params)?;
            let sq = SubmissionQueue::new(&arena.sq_mmap, &arena.sqes_mmap, &fd.params);
            let cq = CompletionQueue::new(arena.cq_mmap(), &fd.params);
            Ok(Uring { sq, cq, arena })
        }
    }
}

pub type UringIo<'fd, M> = Uring<'fd, Sqe64, Cqe16, M>;

pub type Uring128<'fd, M> = Uring<'fd, Sqe128, Cqe32, M>;

pub type UringMix<'fd, M> = Uring<'fd, SqeMix, CqeMix, M>;

// TODO: UringIo self referential lifetime
// #[derive(Debug)]
// pub struct UringIo<S, C> {
//     pub fd: UringFd<S, C>,
//     pub uring: Uring<'fd, S, C>,
// }
