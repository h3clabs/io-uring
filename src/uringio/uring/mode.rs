use std::sync::atomic::Ordering;

use crate::{
    platform::iouring::IoUringSetupFlags,
    shared::constant::DEFAULT_SQ_POLL_IDLE,
    uringio::{
        completion::entry::Cqe,
        setup_args::SetupArgs,
        submission::{entry::Sqe, queue::SubmissionQueue},
    },
};

#[derive(Debug)]
pub enum Ty {
    Iopoll,
    Sqpoll,
}

/// Mode
pub trait Mode: Sized {
    const TYPE: Ty;

    const SETUP_FLAG: IoUringSetupFlags = match Self::TYPE {
        Ty::Iopoll => IoUringSetupFlags::IOPOLL,
        Ty::Sqpoll => IoUringSetupFlags::SQPOLL,
    };

    fn set_sq_ktail<S>(sq: &mut SubmissionQueue<'_, S, Self>, tail: u32);
}

/// Iopoll
#[derive(Debug)]
pub struct Iopoll;

impl Mode for Iopoll {
    const TYPE: Ty = Ty::Iopoll;

    #[inline]
    fn set_sq_ktail<S>(sq: &mut SubmissionQueue<'_, S, Self>, tail: u32) {
        // SAFETY: No concurrent set ktail in IOPOLL mode
        unsafe { *sq.ktail.as_ptr() = tail }
    }
}

impl Iopoll {
    pub fn new_args<S, C>(entries: u32) -> SetupArgs<S, C, Self>
    where
        S: Sqe,
        C: Cqe,
    {
        SetupArgs::new(entries)
            .setup_iopoll()
            .setup_clamp()
            .setup_submit_all()
            .setup_coop_taskrun()
            .setup_taskrun_flag()
            .setup_single_issuer()
            .setup_defer_taskrun()
            .setup_hybrid_iopoll()
    }
}

/// Sqpoll
#[derive(Debug)]
pub struct Sqpoll;

impl Mode for Sqpoll {
    const TYPE: Ty = Ty::Sqpoll;

    #[inline]
    fn set_sq_ktail<S>(sq: &mut SubmissionQueue<'_, S, Self>, tail: u32) {
        sq.ktail.store(tail, Ordering::Release)
    }
}

impl Sqpoll {
    pub fn new_args<S, C>(entries: u32) -> SetupArgs<S, C, Self>
    where
        S: Sqe,
        C: Cqe,
    {
        SetupArgs::new(entries)
            .setup_sqpoll(DEFAULT_SQ_POLL_IDLE)
            .setup_clamp()
            .setup_submit_all()
            .setup_single_issuer()
    }
}
