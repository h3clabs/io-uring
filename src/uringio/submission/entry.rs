use std::{
    any::type_name,
    fmt::{Debug, Formatter},
    marker::PhantomData,
    mem::transmute,
    ops::{Deref, DerefMut},
};

use crate::{
    platform::iouring::{IoUringSetupFlags, IoUringSqe},
    uringio::operator::opcode::Opcode,
};

#[derive(Debug)]
pub enum Ty {
    Sqe64,
    Sqe128,
    SqeMixed,
}

const BASE_SQE_SIZE: usize = size_of::<IoUringSqe>();

pub trait SubmissionEntry {
    const TYPE: Ty;

    const SETUP_FLAG: IoUringSetupFlags = match Self::TYPE {
        Ty::Sqe64 => IoUringSetupFlags::empty(),
        Ty::Sqe128 => IoUringSetupFlags::SQE128,
        Ty::SqeMixed => IoUringSetupFlags::SQE_MIXED,
    };

    const SETUP_SQE_SIZE: usize = match Self::TYPE {
        Ty::Sqe64 | Ty::SqeMixed => BASE_SQE_SIZE,
        Ty::Sqe128 => BASE_SQE_SIZE * 2,
    };
}

/// Sqe64
#[repr(transparent)]
pub struct Sqe64 {
    sqe: IoUringSqe,
}

impl SubmissionEntry for Sqe64 {
    const TYPE: Ty = Ty::Sqe64;
}

impl Debug for Sqe64 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: struct detail
        f.debug_struct(type_name::<Self>()).finish()
    }
}

impl Sqe64 {
    pub const fn new(sqe: IoUringSqe) -> Self {
        Self { sqe }
    }
}

impl Deref for Sqe64 {
    type Target = IoUringSqe;

    fn deref(&self) -> &Self::Target {
        &self.sqe
    }
}

impl DerefMut for Sqe64 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sqe
    }
}

/// Sqe128
#[repr(C)]
pub struct Sqe128 {
    sqe: IoUringSqe,
    extra_data: [u8; 64],
}

impl SubmissionEntry for Sqe128 {
    const TYPE: Ty = Ty::Sqe128;
}

impl Debug for Sqe128 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: struct detail
        f.debug_struct(type_name::<Self>()).finish()
    }
}

impl Sqe128 {
    pub const fn new(sqe: IoUringSqe) -> Self {
        Self { sqe, extra_data: [0; 64] }
    }

    #[inline]
    pub const fn uring_cmd(&mut self) -> &mut [u8; 80] {
        unsafe { transmute(&mut self.sqe.addr3_or_cmd) }
    }
}

impl Deref for Sqe128 {
    type Target = IoUringSqe;

    fn deref(&self) -> &Self::Target {
        &self.sqe
    }
}

impl DerefMut for Sqe128 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sqe
    }
}

/// SqeMixed
#[repr(transparent)]
pub struct SqeMix {
    sqe: IoUringSqe,
    extra_data: PhantomData<[u8; 64]>,
}

impl SubmissionEntry for SqeMix {
    const TYPE: Ty = Ty::SqeMixed;
}

impl Debug for SqeMix {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO: struct detail
        f.debug_struct(type_name::<Self>()).finish()
    }
}

impl SqeMix {
    #[inline]
    pub fn is_sqe128(&self) -> bool {
        self.sqe.opcode.is_sqe128()
    }

    #[inline]
    pub const unsafe fn uring_cmd(&mut self) -> &mut [u8; 80] {
        transmute(&mut self.sqe.addr3_or_cmd.cmd)
    }
}

impl Deref for SqeMix {
    type Target = IoUringSqe;

    fn deref(&self) -> &Self::Target {
        &self.sqe
    }
}

impl DerefMut for SqeMix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sqe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sqe_size() {
        assert_eq!(Sqe64::SETUP_SQE_SIZE, 64);
        assert_eq!(Sqe128::SETUP_SQE_SIZE, 128);
        assert_eq!(SqeMix::SETUP_SQE_SIZE, 64);
    }

    #[test]
    fn test_entry_size() {
        assert_eq!(size_of::<Sqe64>(), 64);
        assert_eq!(size_of::<Sqe128>(), 128);
        assert_eq!(size_of::<SqeMix>(), 64);
    }
}
