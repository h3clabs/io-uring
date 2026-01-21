use std::{cmp, marker::PhantomData, os::fd::AsFd};

use crate::{
    platform::{
        iouring::{
            IoUringFeatureFlags, OwnedFd, IOURING_OFF_CQ_RING, IOURING_OFF_SQES,
            IOURING_OFF_SQ_RING,
        },
        mmap::Mmap,
    },
    shared::error::Result,
    uringio::{completion::entry::Cqe, submission::entry::Sqe, uring::args::UringArgs},
};

/// MmapArena
#[derive(Debug)]
pub struct MmapArena<'fd, S, C, M> {
    pub sq_mmap: Mmap,
    pub sqes_mmap: Mmap,
    cq_mmap: Option<Mmap>,

    _marker_: PhantomData<(&'fd OwnedFd, S, C, M)>,
}

impl<'fd, S, C, M> MmapArena<'fd, S, C, M>
where
    S: Sqe,
    C: Cqe,
{
    pub fn new<Fd>(fd: &Fd, args: &UringArgs<S, C, M>) -> Result<Self>
    where
        Fd: AsFd,
    {
        let sq_size = args.sq_size();
        let cq_size = args.cq_size();

        let sqes_size = args.sqes_size();
        let sqes_mmap = Mmap::new(fd, sqes_size, IOURING_OFF_SQES)?;

        if args.features.contains(IoUringFeatureFlags::SINGLE_MMAP) {
            let mm_size = cmp::max(sq_size, cq_size);
            let sq_mmap = Mmap::new(fd, mm_size, IOURING_OFF_SQ_RING)?;
            Ok(Self { sq_mmap, sqes_mmap, cq_mmap: None, _marker_: PhantomData })
        } else {
            let sq_mmap = Mmap::new(fd, sq_size, IOURING_OFF_SQ_RING)?;
            let cq_mmap = Mmap::new(fd, cq_size, IOURING_OFF_CQ_RING)?;
            Ok(Self { sq_mmap, sqes_mmap, cq_mmap: Some(cq_mmap), _marker_: PhantomData })
        }
    }

    #[inline]
    pub const fn cq_mmap(&self) -> &Mmap {
        match &self.cq_mmap {
            Some(cq_mmap) => cq_mmap,
            None => &self.sq_mmap,
        }
    }
}
