use crate::uringio::{
    completion::{entry::Cqe, queue::CompletionQueue},
    ring::mode::Mode,
};

#[derive(Debug)]
pub struct Collector<'c, 'fd, C, M> {
    pub(crate) head: u32,
    pub(crate) tail: u32,
    pub queue: &'c mut CompletionQueue<'fd, C, M>,
}

impl<'c, 'fd, C, M> Collector<'c, 'fd, C, M> {
    #[inline]
    pub const fn size(&self) -> u32 {
        self.tail.wrapping_sub(self.head)
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.size() == self.queue.size
    }
}

impl<'c, 'fd, C, M> Collector<'c, 'fd, C, M>
where
    C: Cqe,
    M: Mode,
{
    #[inline]
    pub fn update_head(&mut self) {
        self.queue.set_head(self.head);
    }

    #[inline]
    pub fn update_tail(&mut self) {
        self.tail = self.queue.tail();
    }

    pub fn update(&mut self) {
        self.update_head();
        self.update_tail();
    }
}

impl<'c, 'fd, C, M> Iterator for Collector<'c, 'fd, C, M> {
    type Item = &'c C;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.head == self.tail {
            return None;
        }

        let cqe = self.queue.get_cqe(self.head);
        self.head = self.head.wrapping_add(1);
        Some(unsafe { cqe.as_ref() })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.size() as usize;
        (size, Some(size))
    }
}

impl<'c, 'fd, C, M> ExactSizeIterator for Collector<'c, 'fd, C, M> {
    #[inline]
    fn len(&self) -> usize {
        self.size() as usize
    }
}
