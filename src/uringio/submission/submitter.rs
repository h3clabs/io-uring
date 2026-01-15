use std::io::Result;

use crate::{
    shared::null::{Null, NULL},
    uringio::{
        operator::Op,
        submission::{
            entry::{Sqe128, Sqe64, SqeMix},
            queue::SubmissionQueue,
        },
    },
};

pub trait Submit<T> {
    fn submit(&mut self, op: T) -> Result<Null>;
}

#[derive(Debug)]
pub struct Submitter<'s, 'fd, S, M> {
    pub tail: u32,
    pub queue: &'s mut SubmissionQueue<'fd, S, M>,
}

impl<'s, 'fd, T, M> Submit<T> for Submitter<'s, 'fd, Sqe64, M>
where
    T: Op<Entry = Sqe64>,
{
    fn submit(&mut self, op: T) -> Result<Null> {
        let sqe = T::Entry::from(op);
        todo!();
        Ok(NULL)
    }
}

impl<'s, 'fd, T, M> Submit<T> for Submitter<'s, 'fd, Sqe128, M>
where
    T: Op<Entry = Sqe128>,
{
    fn submit(&mut self, op: T) -> Result<Null> {
        let sqe = T::Entry::from(op);
        todo!();
        Ok(NULL)
    }
}

impl<'s, 'fd, T, M> Submit<T> for Submitter<'s, 'fd, SqeMix, M>
where
    T: Op<Entry = Sqe64>,
{
    fn submit(&mut self, op: T) -> Result<Null> {
        let sqe = T::Entry::from(op);
        todo!();
        Ok(NULL)
    }
}

// submitter
// collector

// FIX: conflicting implementations
pub trait SubmitCmd<T> {
    fn submit(&mut self, op: T) -> Result<Null>;
}

impl<'s, 'fd, T, M> SubmitCmd<T> for Submitter<'s, 'fd, SqeMix, M>
where
    T: Op<Entry = Sqe128>,
{
    fn submit(&mut self, op: T) -> Result<Null> {
        let sqe = T::Entry::from(op);
        todo!();
        Ok(NULL)
    }
}
