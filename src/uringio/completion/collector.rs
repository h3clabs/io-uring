use crate::uringio::{
    completion::{entry::Cqe, queue::CompletionQueue},
    uring::mode::Mode,
};

#[derive(Debug)]
pub struct Collector<'c, 'fd, C, M> {
    pub(crate) head: u32,
    pub(crate) tail: u32,
    pub queue: &'c mut CompletionQueue<'fd, C, M>,
}

impl<'c, 'fd, C, M> Collector<'c, 'fd, C, M>
where
    C: Cqe,
    M: Mode,
{
    pub fn pop(&mut self) -> Option<C> {
        // if self.is_full() {
        //     return Err(sqe)
        // }

        // self.queue[self.tail] = sqe.into();
        // self.tail = self.tail.wrapping_add(1);
        None
    }

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
    pub const fn size(&self) -> u32 {
        self.tail.wrapping_sub(self.head)
    }

    #[inline]
    pub const fn is_full(&self) -> bool {
        self.size() == self.queue.size
    }
}
