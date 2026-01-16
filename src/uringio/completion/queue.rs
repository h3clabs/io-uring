use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    platform::{iouring::IoUringParams, mmap::Mmap},
    uringio::{
        completion::{collector::Collector, entry::Cqe},
        uring::mode::Mode,
    },
};

/// CompletionQueue
#[derive(Debug)]
pub struct CompletionQueue<'fd, C, M> {
    pub khead: &'fd AtomicU32,
    pub ktail: &'fd AtomicU32,
    pub mask: u32,
    pub size: u32,
    pub overflow: &'fd AtomicU32,
    pub cqes: NonNull<C>,
    pub flags: &'fd AtomicU32,

    _marker_: PhantomData<M>,
}

impl<'fd, C, M> CompletionQueue<'fd, C, M> {
    pub const unsafe fn new(cq_mmap: &Mmap, params: &IoUringParams) -> Self {
        let IoUringParams { cq_off, .. } = params;

        let khead = cq_mmap.offset(cq_off.head).cast().as_ref();
        let ktail = cq_mmap.offset(cq_off.tail).cast().as_ref();
        let mask = cq_mmap.offset(cq_off.ring_mask).cast().read();
        let size = cq_mmap.offset(cq_off.ring_entries).cast().read();
        let overflow = cq_mmap.offset(cq_off.overflow).cast().as_ref();
        let cqes = cq_mmap.offset(cq_off.cqes).cast();
        let flags = cq_mmap.offset(cq_off.flags).cast().as_ref();

        Self { khead, ktail, mask, size, overflow, cqes, flags, _marker_: PhantomData }
    }

    #[inline]
    pub const fn get_cqe(&self, idx: u32) -> NonNull<C> {
        // SAFETY: index masked
        unsafe { self.cqes.add((idx & self.mask) as usize) }
    }
}

impl<'fd, C, M> CompletionQueue<'fd, C, M>
where
    C: Cqe,
    M: Mode,
{
    #[inline]
    pub const fn head(&self) -> u32 {
        // SAFETY: userspace set CompletionQueue khead
        unsafe { *self.khead.as_ptr() }
    }

    #[inline]
    pub fn tail(&self) -> u32 {
        // TODO: Relaxed or Read ptr?
        self.ktail.load(Ordering::Acquire)
    }

    #[inline]
    pub fn set_head(&mut self, head: u32) {
        self.khead.store(head, Ordering::Release);
    }

    #[inline]
    pub fn collector(&mut self) -> Collector<'_, 'fd, C, M> {
        Collector { head: self.head(), tail: self.tail(), queue: self }
    }
}

impl<'fd, C, M> Index<u32> for CompletionQueue<'fd, C, M> {
    type Output = C;

    #[inline]
    fn index(&self, index: u32) -> &Self::Output {
        // TODO: handle SQARRAY SubmissionIndex
        unsafe { self.get_cqe(index).as_ref() }
    }
}

impl<'fd, C, M> IndexMut<u32> for CompletionQueue<'fd, C, M> {
    #[inline]
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        unsafe { self.get_cqe(index).as_mut() }
    }
}
