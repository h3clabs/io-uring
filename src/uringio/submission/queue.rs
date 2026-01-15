use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    platform::{
        iouring::{IoUringParams, IoUringSetupFlags},
        mmap::Mmap,
    },
    uringio::{
        submission::{entry::SubmissionEntry, submitter::Submitter},
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
    pub indices: NonNull<u32>,
    pub sqes: NonNull<S>,

    _marker_: PhantomData<M>,
}

impl<'fd, S, M> SubmissionQueue<'fd, S, M>
where
    S: SubmissionEntry,
    M: Mode,
{
    pub const unsafe fn new(sq_mmap: &Mmap, sqe_mmap: &Mmap, params: &IoUringParams) -> Self {
        let IoUringParams { sq_off, .. } = params;

        let khead = sq_mmap.offset(sq_off.head).cast().as_ref();
        let ktail = sq_mmap.offset(sq_off.tail).cast().as_ref();
        let mask = sq_mmap.offset(sq_off.ring_mask).cast().read();
        let size = sq_mmap.offset(sq_off.ring_entries).cast().read();
        let flags = sq_mmap.offset(sq_off.flags).cast().as_ref();
        let dropped = sq_mmap.offset(sq_off.dropped).cast().as_ref();
        let indices = sq_mmap.offset(sq_off.array).cast();
        let sqes = sqe_mmap.ptr.cast();

        Self { khead, ktail, mask, size, flags, dropped, indices, sqes, _marker_: PhantomData }
    }

    #[inline]
    pub fn head(&self) -> u32 {
        self.khead.load(Ordering::Acquire)
    }

    #[inline]
    pub fn tail(&self) -> u32 {
        // SAFETY: userspace set SubmissionQueue ktail
        unsafe { *self.ktail.as_ptr() }
    }

    pub fn set_tail(&mut self, tail: u32) {
        M::set_sq_ktail(self, tail);
    }

    pub fn submitter(&mut self) -> Submitter<'_, 'fd, S, M> {
        Submitter { tail: self.tail(), queue: self }
    }

    #[inline]
    pub const fn size(&self) -> usize {
        self.size as usize
    }
}

/// SubmissionIndex
#[derive(Debug, Default)]
pub struct SubmissionIndex<'a> {
    slice: &'a mut [u32],
}

impl<'a> SubmissionIndex<'a> {
    pub unsafe fn new(sq_mmap: &Mmap, params: &IoUringParams) -> Self {
        let IoUringParams { sq_off, flags, .. } = params;

        if flags.contains(IoUringSetupFlags::NO_SQARRAY) {
            return Self::default();
        }

        let size = sq_mmap.offset(sq_off.ring_entries).cast::<u32>().read();
        let head = sq_mmap.offset(sq_off.array).cast::<u32>();

        let slice = NonNull::slice_from_raw_parts(head, size as usize).as_mut();
        for idx in 0..slice.len() {
            slice[idx] = idx as u32;
        }

        Self { slice }
    }
}

impl<'a> Deref for SubmissionIndex<'a> {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        self.slice
    }
}

impl<'a> DerefMut for SubmissionIndex<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.slice
    }
}
