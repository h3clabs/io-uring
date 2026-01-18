pub mod desc;
pub mod mode;

use crate::{
    platform::iouring::IoUringParams,
    shared::error::Result,
    uringio::{
        completion::{
            collector::Collector,
            entry::{Cqe, Cqe16, Cqe32, CqeMix},
            queue::CompletionQueue,
        },
        mmap_arena::MmapArena,
        submission::{
            entry::{Sqe, Sqe128, Sqe64, SqeMix},
            queue::SubmissionQueue,
            submitter::Submitter,
        },
        uring::mode::Mode,
    },
};

/// Uring
#[derive(Debug)]
pub struct Uring<'fd, S, C, M> {
    pub sq: SubmissionQueue<'fd, S, M>,
    pub cq: CompletionQueue<'fd, C, M>,
}

impl<'fd, S, C, M> Uring<'fd, S, C, M>
where
    S: Sqe,
    C: Cqe,
    M: Mode,
{
    pub fn new(arena: &'fd MmapArena<S, C, M>, params: &IoUringParams) -> Result<Self> {
        unsafe {
            let sq = SubmissionQueue::new(&arena.sq_mmap, &arena.sqes_mmap, params);
            let cq = CompletionQueue::new(arena.cq_mmap(), params);
            Ok(Uring { sq, cq })
        }
    }

    pub fn submitter(&mut self) -> Submitter<'_, 'fd, S, M> {
        self.sq.submitter()
    }

    pub fn collector(&mut self) -> Collector<'_, 'fd, C, M> {
        self.cq.collector()
    }

    pub fn borrow(&mut self) -> (Submitter<'_, 'fd, S, M>, Collector<'_, 'fd, C, M>) {
        (self.sq.submitter(), self.cq.collector())
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
