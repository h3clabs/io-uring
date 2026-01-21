use crate::uringio::{completion::queue::CompletionQueue, uring::mode::Mode};

#[derive(Debug)]
pub struct Collector<'c, 'fd, C, M>
where
    M: Mode,
{
    pub(crate) head: u32,
    pub(crate) tail: u32,
    pub queue: &'c mut CompletionQueue<'fd, C, M>,
}

impl<'c, 'fd, C, M> Collector<'c, 'fd, C, M>
where
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

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.size() == self.queue.size
    }

    #[inline]
    pub const fn size(&self) -> u32 {
        self.tail.wrapping_sub(self.head)
    }
}

impl<'c, 'fd, C, M> Drop for Collector<'c, 'fd, C, M>
where
    M: Mode,
{
    fn drop(&mut self) {
        self.update_head();
    }
}

impl<'c, 'fd, C, M> Iterator for Collector<'c, 'fd, C, M>
where
    M: Mode,
{
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

impl<'c, 'fd, C, M> ExactSizeIterator for Collector<'c, 'fd, C, M>
where
    M: Mode,
{
    #[inline]
    fn len(&self) -> usize {
        self.size() as usize
    }
}
