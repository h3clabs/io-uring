use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    platform::{iouring::IoUringParams, mmap::Mmap},
    uringio::{
        submission::{entry::Sqe, index::SubmissionIndex, submitter::Submitter},
        uring::mode::Mode,
    },
};

/// SubmissionQueue
#[derive(Debug)]
pub struct SubmissionQueue<'fd, S, M> {
    pub khead: &'fd AtomicU32,
    pub ktail: &'fd AtomicU32,
    pub mask: u32,
    pub size: u32,
    pub flags: &'fd AtomicU32,
    pub dropped: &'fd AtomicU32,
    pub sqes: NonNull<S>,

    _marker_: PhantomData<M>,
}

impl<'fd, S, M> SubmissionQueue<'fd, S, M> {
    pub unsafe fn new(sq_mmap: &Mmap, sqe_mmap: &Mmap, params: &IoUringParams) -> Self {
        let IoUringParams { sq_off, .. } = params;

        let khead = sq_mmap.offset(sq_off.head).cast().as_ref();
        let ktail = sq_mmap.offset(sq_off.tail).cast().as_ref();
        let mask = sq_mmap.offset(sq_off.ring_mask).cast().read();
        let size = sq_mmap.offset(sq_off.ring_entries).cast().read();
        let flags = sq_mmap.offset(sq_off.flags).cast().as_ref();
        let dropped = sq_mmap.offset(sq_off.dropped).cast().as_ref();
        let sqes = sqe_mmap.ptr.cast();
        SubmissionIndex::setup(sq_mmap, params);

        Self { khead, ktail, mask, size, flags, dropped, sqes, _marker_: PhantomData }
    }

    #[inline]
    pub const fn get_sqe(&self, idx: u32) -> NonNull<S> {
        // SAFETY: index masked
        unsafe { self.sqes.add((idx & self.mask) as usize) }
    }
}

impl<'fd, S, M> SubmissionQueue<'fd, S, M>
where
    S: Sqe,
    M: Mode,
{
    #[inline]
    pub fn head(&self) -> u32 {
        self.khead.load(Ordering::Acquire)
    }

    #[inline]
    pub const fn tail(&self) -> u32 {
        // SAFETY: userspace set SubmissionQueue ktail
        unsafe { *self.ktail.as_ptr() }
    }

    #[inline]
    pub fn set_tail(&mut self, tail: u32) {
        M::set_sq_ktail(self, tail);
    }

    #[inline]
    pub fn submitter(&mut self) -> Submitter<'_, 'fd, S, M> {
        Submitter { head: self.head(), tail: self.tail(), queue: self }
    }
}

impl<'fd, S, M> Index<u32> for SubmissionQueue<'fd, S, M> {
    type Output = S;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        // TODO: handle SQARRAY SubmissionIndex
        unsafe { self.get_sqe(index).as_ref() }
    }
}

impl<'fd, S, M> IndexMut<u32> for SubmissionQueue<'fd, S, M> {
    #[inline]
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        unsafe { self.get_sqe(index).as_mut() }
    }
}
