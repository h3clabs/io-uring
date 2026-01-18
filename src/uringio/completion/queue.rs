use std::{
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr::NonNull,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::{
    platform::{
        iouring::{IoUringCqFlags, IoUringParams},
        mmap::Mmap,
    },
    uringio::{
        completion::{collector::Collector, entry::Cqe},
        uring::mode::Mode,
    },
};

/// CompletionQueue
#[derive(Debug)]
pub struct CompletionQueue<'fd, C, M> {
    pub cqes: NonNull<C>,
    pub k_head: &'fd AtomicU32,
    pub k_tail: &'fd AtomicU32,
    pub mask: u32,
    pub size: u32,
    pub k_flags: &'fd AtomicU32,
    pub k_overflow: &'fd AtomicU32,

    _marker_: PhantomData<M>,
}

impl<'fd, C, M> CompletionQueue<'fd, C, M> {
    pub const unsafe fn new(cq_mmap: &Mmap, params: &IoUringParams) -> Self {
        let IoUringParams { cq_off, .. } = params;

        let cqes = cq_mmap.offset(cq_off.cqes).cast();
        let k_head = cq_mmap.offset(cq_off.head).cast().as_ref();
        let k_tail = cq_mmap.offset(cq_off.tail).cast().as_ref();
        let mask = cq_mmap.offset(cq_off.ring_mask).cast().read();
        let size = cq_mmap.offset(cq_off.ring_entries).cast().read();
        let k_flags = cq_mmap.offset(cq_off.flags).cast().as_ref();
        let k_overflow = cq_mmap.offset(cq_off.overflow).cast().as_ref();

        Self { cqes, k_head, k_tail, mask, size, k_flags, k_overflow, _marker_: PhantomData }
    }

    pub fn flags(&self, order: Ordering) -> IoUringCqFlags {
        let bits = self.k_flags.load(order);
        IoUringCqFlags::from_bits_retain(bits)
    }

    pub fn overflow(&self) -> u32 {
        self.k_overflow.load(Ordering::Acquire)
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
        unsafe { *self.k_head.as_ptr() }
    }

    #[inline]
    pub fn tail(&self) -> u32 {
        // TODO: Relaxed or Read ptr?
        self.k_tail.load(Ordering::Acquire)
    }

    #[inline]
    pub fn set_head(&mut self, head: u32) {
        self.k_head.store(head, Ordering::Release);
    }

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
