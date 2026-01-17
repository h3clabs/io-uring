use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    platform::{
        iouring::{IoUringParams, IoUringSqFlags},
        mmap::Mmap,
    },
    uringio::{
        ring::mode::Mode,
        submission::{entry::Sqe, index::SubmissionIndex, submitter::Submitter},
    },
};

/// SubmissionQueue
#[derive(Debug)]
pub struct SubmissionQueue<'fd, S, M> {
    pub sqes: NonNull<S>,
    pub k_head: &'fd AtomicU32,
    pub k_tail: &'fd AtomicU32,
    pub mask: u32,
    pub size: u32,
    pub k_flags: &'fd AtomicU32,
    pub k_dropped: &'fd AtomicU32,

    _marker_: PhantomData<M>,
}

impl<'fd, S, M> SubmissionQueue<'fd, S, M> {
    pub unsafe fn new(sq_mmap: &Mmap, sqe_mmap: &Mmap, params: &IoUringParams) -> Self {
        let IoUringParams { sq_off, .. } = params;

        let sqes = sqe_mmap.ptr().cast();
        let k_head = sq_mmap.offset(sq_off.head).cast().as_ref();
        let k_tail = sq_mmap.offset(sq_off.tail).cast().as_ref();
        let mask = sq_mmap.offset(sq_off.ring_mask).cast().read();
        let size = sq_mmap.offset(sq_off.ring_entries).cast().read();
        let k_flags = sq_mmap.offset(sq_off.flags).cast().as_ref();
        let k_dropped = sq_mmap.offset(sq_off.dropped).cast().as_ref();
        SubmissionIndex::setup(sq_mmap, params);

        Self { sqes, k_head, k_tail, mask, size, k_flags, k_dropped, _marker_: PhantomData }
    }

    pub fn flags(&self) -> IoUringSqFlags {
        let bits = self.k_flags.load(Ordering::Acquire);
        IoUringSqFlags::from_bits_retain(bits)
    }

    pub fn dropped(&self) -> u32 {
        self.k_dropped.load(Ordering::Acquire)
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
        self.k_head.load(Ordering::Acquire)
    }

    #[inline]
    pub const fn tail(&self) -> u32 {
        // SAFETY: userspace set SubmissionQueue ktail
        unsafe { *self.k_tail.as_ptr() }
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
