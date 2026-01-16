use std::{cmp, marker::PhantomData, os::fd::AsFd};

use crate::{
    platform::{
        iouring::{
            IoUringFeatureFlags, IoUringParams, OwnedFd, IOURING_OFF_CQ_RING, IOURING_OFF_SQES,
            IOURING_OFF_SQ_RING,
        },
        mmap::Mmap,
    },
    shared::error::Result,
    uringio::{completion::entry::Cqe, setup_args::ParamsExt, submission::entry::Sqe},
};

/// MmapArena
#[derive(Debug)]
pub struct MmapArena<'fd, S, C> {
    pub sq_mmap: Mmap,
    pub sqes_mmap: Mmap,
    cq_mmap: Option<Mmap>,

    _marker_: PhantomData<(&'fd OwnedFd, S, C)>,
}

impl<'fd, S, C> MmapArena<'fd, S, C>
where
    S: Sqe,
    C: Cqe,
{
    pub fn new<Fd>(fd: &'fd Fd, params: &IoUringParams) -> Result<Self>
    where
        Fd: AsFd,
    {
        let sq_size = ParamsExt::<S, C>::sq_size(params);
        let cq_size = ParamsExt::<S, C>::cq_size(params);

        let sqes_size = ParamsExt::<S, C>::sqes_size(params);
        let sqes_mmap = Mmap::new(fd, sqes_size, IOURING_OFF_SQES)?;

        if params.features.contains(IoUringFeatureFlags::SINGLE_MMAP) {
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
