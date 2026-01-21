use std::{
    io::{Error, ErrorKind, Result},
    marker::PhantomData,
    sync::{atomic, atomic::Ordering},
};

use crate::{
    platform::iouring::{
        io_uring_enter, io_uring_register, AsFd, AsRawFd, BorrowedFd, IoUringEnterFlags,
        IoUringFeatureFlags,
        IoUringRegisterOp::{RegisterRingFds, UnregisterRingFds},
        IoUringRsrcUpdate, IoUringSqFlags, OwnedFd,
    },
    shared::null::{Null, NULL},
    uringio::{
        register::args::{RegisterArgs, RegisterRingFd},
        submission::submitter::Submitter,
        uring::{
            args::UringArgs,
            mode::{Iopoll, Mode, Sqpoll},
        },
    },
};

#[derive(Debug)]
pub struct UringHandler<'fd, S, C, M> {
    enter_fd: BorrowedFd<'fd>,
    // TODO: init flags
    enter_flags: IoUringEnterFlags,
    features: IoUringFeatureFlags,

    _marker_: PhantomData<(S, C, M)>,
}

impl<'fd, S, C, M> UringHandler<'fd, S, C, M>
where
    M: Mode,
{
    pub fn new(fd: &'fd OwnedFd, args: &UringArgs<S, C, M>) -> Self {
        Self {
            enter_fd: fd.as_fd(),
            enter_flags: M::ENTER_FLAG,
            features: args.features,
            _marker_: PhantomData,
        }
    }
}

impl<'fd, S, C, M> UringHandler<'fd, S, C, M> {
    #[inline]
    pub fn features(&self) -> &IoUringFeatureFlags {
        &self.features
    }

    #[inline]
    pub fn is_ring_registered(&self) -> bool {
        self.enter_flags.contains(IoUringEnterFlags::REGISTERED_RING)
    }

    pub fn register_ring_fd(&mut self) -> Result<Null> {
        #[cfg(feature = "features-checker")]
        {
            if !self.features.contains(IoUringFeatureFlags::REG_REG_RING) {
                return Err(Error::new(ErrorKind::Other, "Feature REG_REG_RING Invalid"));
            }
        }

        if self.is_ring_registered() {
            return Err(Error::new(ErrorKind::Other, "Ring fd registered"));
        }

        #[allow(unused_mut)]
        let mut args = IoUringRsrcUpdate::new(self.enter_fd.as_raw_fd());
        // SAFETY: asm options !readonly
        let num = unsafe { io_uring_register(&self.enter_fd, RegisterRingFds, args.as_ptr(), 1)? };

        if num != 1 {
            return Err(Error::new(ErrorKind::Other, "Failed to register ring fd"));
        }

        self.enter_fd = unsafe { BorrowedFd::borrow_raw(args.offset as _) };
        self.enter_flags |= IoUringEnterFlags::REGISTERED_RING;
        Ok(NULL)
    }

    #[inline]
    pub fn enter(
        &self,
        to_submit: u32,
        min_complete: u32,
        flags: IoUringEnterFlags,
    ) -> Result<u32> {
        Ok(unsafe {
            io_uring_enter(self.enter_fd, to_submit, min_complete, self.enter_flags | flags)?
        })
    }

    pub fn get_events(&self) -> Result<u32> {
        self.enter(0, 0, IoUringEnterFlags::GETEVENTS)
    }
}

impl<'fd, S, C> UringHandler<'fd, S, C, Iopoll> {
    pub fn submit(
        &mut self,
        submitter: &mut Submitter<'_, 'fd, S, Iopoll>,
        min_complete: u32,
    ) -> Result<u32> {
        self.enter(submitter.size(), min_complete, IoUringEnterFlags::GETEVENTS)
    }
}

impl<'fd, S, C> UringHandler<'fd, S, C, Sqpoll> {
    pub fn sq_wait(&self) -> Result<u32> {
        self.enter(0, 0, IoUringEnterFlags::SQ_WAIT)
    }

    pub fn submit(
        &mut self,
        submitter: &mut Submitter<'_, 'fd, S, Sqpoll>,
        min_complete: u32,
    ) -> Result<u32> {
        let mut flags = IoUringEnterFlags::default();

        // TODO: void fence(SeqCst): https://github.com/axboe/liburing/issues/541
        atomic::fence(Ordering::SeqCst);
        let sq_flags = submitter.queue.flags(Ordering::Relaxed);

        if sq_flags.contains(IoUringSqFlags::NEED_WAKEUP) {
            flags.insert(IoUringEnterFlags::SQ_WAKEUP);
        }

        if min_complete > 0 || sq_flags.contains(IoUringSqFlags::CQ_OVERFLOW) {
            // IORING_ENTER_GETEVENTS call io_cqring_do_overflow_flush()
            flags.insert(IoUringEnterFlags::GETEVENTS);
        }

        self.enter(submitter.size(), min_complete, flags)
    }
}

impl<'fd, S, C, M> Drop for UringHandler<'fd, S, C, M> {
    fn drop(&mut self) {
        if self.is_ring_registered() {
            let idx = self.enter_fd.as_raw_fd() as u32;
            let args = IoUringRsrcUpdate::unregister(idx);
            unsafe {
                // Error ignored
                let _ = io_uring_register(&self.enter_fd, UnregisterRingFds, args.as_ptr(), 0);
            }
        }
    }
}
