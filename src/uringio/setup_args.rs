use std::marker::PhantomData;

use crate::{
    platform::iouring::{io_uring_setup, AsRawFd, IoUringParams, IoUringSetupFlags},
    shared::error::Result,
    uringio::{
        completion::entry::Cqe,
        mmap_arena::MmapArena,
        submission::entry::Sqe,
        uring::{desc::RingDesc, mode::Mode},
    },
};

#[derive(Debug)]
#[repr(transparent)]
pub struct SetupArgs<S, C, M> {
    pub params: IoUringParams,

    _marker_: PhantomData<(S, C, M)>,
}

impl<S, C, M> SetupArgs<S, C, M>
where
    S: Sqe,
    C: Cqe,
    M: Mode,
{
    pub fn new(entries: u32) -> Self {
        let mut params = IoUringParams::default();
        params.flags |= S::SETUP_FLAG;
        params.flags |= C::SETUP_FLAG;
        params.flags |= M::SETUP_FLAG;
        params.sq_entries = entries;
        SetupArgs { params, _marker_: PhantomData }
    }

    pub fn sqsize(mut self, entries: u32) -> Self {
        self.params.sq_entries = entries;
        self
    }

    pub fn iopoll(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::IOPOLL;
        self
    }

    pub fn sqpoll(mut self, idle: u32) -> Self {
        self.params.flags |= IoUringSetupFlags::SQPOLL;
        self.params.sq_thread_idle = idle;
        self
    }

    pub fn sqpoll_cpu(mut self, cpu: u32) -> Self {
        self.params.flags |= IoUringSetupFlags::SQ_AFF;
        self.params.sq_thread_cpu = cpu;
        self
    }

    pub fn cqsize(mut self, entries: u32) -> Self {
        self.params.flags |= IoUringSetupFlags::CQSIZE;
        self.params.cq_entries = entries;
        self
    }

    pub fn clamp(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::CLAMP;
        self
    }

    pub fn attach_wq<F>(mut self, fd: F) -> Self
    where
        F: AsRawFd,
    {
        self.params.flags |= IoUringSetupFlags::ATTACH_WQ;
        self.params.wq_fd = fd.as_raw_fd();
        self
    }

    pub fn r_disabled(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::R_DISABLED;
        self
    }

    pub fn submit_all(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::SUBMIT_ALL;
        self
    }

    // Must use with SQPOLL
    pub fn coop_taskrun(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::COOP_TASKRUN;
        self
    }

    // Must use with SQPOLL
    pub fn taskrun_flag(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::TASKRUN_FLAG;
        self
    }

    pub fn single_issuer(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::SINGLE_ISSUER;
        self
    }

    // Must use with IOPOLL | SINGLE_ISSUER
    pub fn defer_taskrun(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::DEFER_TASKRUN;
        self
    }

    pub fn no_mmap(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::NO_MMAP;
        // TODO: setup hugepage mmap
        self
    }

    // Must use with NO_MMAP
    pub fn registered_fd_only(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::REGISTERED_FD_ONLY;
        self
    }

    pub fn no_sqarray(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::NO_SQARRAY;
        self
    }

    // Must use with IOPOLL
    pub fn hybrid_iopoll(mut self) -> Self {
        self.params.flags |= IoUringSetupFlags::HYBRID_IOPOLL;
        self
    }

    pub fn setup(self) -> Result<(RingDesc<S, C, M>, IoUringParams)> {
        let SetupArgs { mut params, .. } = self;
        let fd = unsafe { io_uring_setup(params.sq_entries, &mut params)? };
        let arena = MmapArena::new(&fd, &params)?;
        Ok((RingDesc::new(fd, arena), params))
    }
}

pub trait ParamsExt<S, C> {
    fn sq_size(&self) -> usize;

    fn sq_indices_size(&self) -> usize;

    fn sqes_size(&self) -> usize;

    fn cq_size(&self) -> usize;

    fn cqes_size(&self) -> usize;
}

impl<S, C> ParamsExt<S, C> for IoUringParams
where
    S: Sqe,
    C: Cqe,
{
    fn sq_size(&self) -> usize {
        self.sq_off.array as usize + <Self as ParamsExt<S, C>>::sq_indices_size(self)
    }

    fn sq_indices_size(&self) -> usize {
        if self.flags.contains(IoUringSetupFlags::NO_SQARRAY) {
            0
        } else {
            self.sq_entries as usize * size_of::<u32>()
        }
    }

    fn sqes_size(&self) -> usize {
        self.sq_entries as usize * S::SETUP_SQE_SIZE
    }

    fn cq_size(&self) -> usize {
        self.cq_off.cqes as usize + <Self as ParamsExt<S, C>>::cqes_size(self)
    }

    fn cqes_size(&self) -> usize {
        self.cq_entries as usize * C::SETUP_CQE_SIZE
    }
}
