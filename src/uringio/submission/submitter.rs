use crate::{
    shared::{
        error::Result,
        null::{Null, NULL},
    },
    uringio::{
        operator::{nop::Nop128, Op},
        ring::mode::Mode,
        submission::{
            entry::{FixSqe, Sqe, Sqe128, Sqe64, SqeMix},
            queue::SubmissionQueue,
        },
    },
};

/// Submitter
#[derive(Debug)]
pub struct Submitter<'s, 'fd, S, M> {
    pub(crate) head: u32,
    pub(crate) tail: u32,
    pub queue: &'s mut SubmissionQueue<'fd, S, M>,
}

impl<'s, 'fd, S, M> Submitter<'s, 'fd, S, M>
where
    S: Sqe,
    M: Mode,
{
    fn push_impl<T>(&mut self, sqe: T) -> Result<Null, T>
    where
        T: Into<S> + FixSqe,
    {
        if self.is_full() {
            return Err(sqe)
        }

        self.queue[self.tail] = sqe.into();
        self.tail = self.tail.wrapping_add(1);

        Ok(NULL)
    }

    #[inline]
    pub fn update_head(&mut self) {
        self.head = self.queue.head();
    }

    #[inline]
    pub fn update_tail(&mut self) {
        self.queue.set_tail(self.tail);
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

    pub fn submit() {}
}

pub trait Submit<T> {
    fn push(&mut self, item: T) -> Result<Null, T>;
}

// Submit to Sqe64 Queue
impl<'s, 'fd, M> Submit<Sqe64> for Submitter<'s, 'fd, Sqe64, M>
where
    M: Mode,
{
    fn push(&mut self, sqe: Sqe64) -> Result<Null, Sqe64> {
        self.push_impl(sqe)
    }
}

// Submit to Sqe128 Queue
impl<'s, 'fd, M> Submit<Sqe128> for Submitter<'s, 'fd, Sqe128, M>
where
    M: Mode,
{
    fn push(&mut self, sqe: Sqe128) -> Result<Null, Sqe128> {
        self.push_impl(sqe)
    }
}

// Submit Sqe64 to SqeMix Queue
impl<'s, 'fd, M> Submit<Sqe64> for Submitter<'s, 'fd, SqeMix, M>
where
    M: Mode,
{
    fn push(&mut self, sqe: Sqe64) -> Result<Null, Sqe64> {
        self.push_impl(sqe)
    }
}

// Submit Sqe128 to SqeMix Queue
impl<'s, 'fd, M> Submit<Sqe128> for Submitter<'s, 'fd, SqeMix, M>
where
    M: Mode,
{
    fn push(&mut self, sqe: Sqe128) -> Result<Null, Sqe128> {
        // Sqe128 take 2 slots
        if self.size() + 2 > self.queue.size {
            return Err(sqe)
        }

        // Padding with IORING_OP_NOP128
        if self.tail.wrapping_add(1) & self.queue.mask == 0 {
            if self.size() + 3 > self.queue.size {
                return Err(sqe)
            }
            // Nop128 slot checked
            let _ = self.push(Nop128::new().skip_cqe());
        }

        unsafe { self.queue.get_sqe(self.tail).cast::<Sqe128>().write(sqe) };
        self.tail = self.tail.wrapping_add(2);

        Ok(NULL)
    }
}

// Submit Op
impl<'s, 'fd, T, S, M> Submit<T> for Submitter<'s, 'fd, S, M>
where
    S: Sqe,
    M: Mode,
    T: Op + Into<S>,
{
    fn push(&mut self, op: T) -> Result<Null, T> {
        self.push_impl(op)
    }
}
