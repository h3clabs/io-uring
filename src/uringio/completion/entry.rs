use std::{
    marker::PhantomData,
    mem::transmute,
    ops::{Deref, DerefMut},
};

use crate::platform::iouring::{IoUringCqe, IoUringCqeFlags, IoUringSetupFlags};

#[derive(Debug)]
pub enum Ty {
    Cqe16,
    Cqe32,
    CqeMix,
}

const BASE_CQE_SIZE: usize = size_of::<IoUringCqe>();

pub trait CompletionEntry {
    const TYPE: Ty;

    const SETUP_FLAG: IoUringSetupFlags = match Self::TYPE {
        Ty::Cqe16 => IoUringSetupFlags::empty(),
        Ty::Cqe32 => IoUringSetupFlags::CQE32,
        Ty::CqeMix => IoUringSetupFlags::CQE_MIXED,
    };

    const SETUP_CQE_SIZE: usize = match Self::TYPE {
        Ty::Cqe16 | Ty::CqeMix => BASE_CQE_SIZE,
        Ty::Cqe32 => BASE_CQE_SIZE * 2,
    };
}

/// Cqe16
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Cqe16 {
    cqe: IoUringCqe,
}

impl CompletionEntry for Cqe16 {
    const TYPE: Ty = Ty::Cqe16;
}

impl Deref for Cqe16 {
    type Target = IoUringCqe;

    fn deref(&self) -> &Self::Target {
        &self.cqe
    }
}

impl DerefMut for Cqe16 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cqe
    }
}

/// Cqe32
#[derive(Debug, Default)]
#[repr(C)]
pub struct Cqe32 {
    cqe: IoUringCqe,
    extra_data: [u64; 2],
}

impl CompletionEntry for Cqe32 {
    const TYPE: Ty = Ty::Cqe32;
}

impl Cqe32 {
    pub const fn extra_data(&self) -> &[u64; 2] {
        &self.extra_data
    }
}

impl Deref for Cqe32 {
    type Target = IoUringCqe;

    fn deref(&self) -> &Self::Target {
        &self.cqe
    }
}

impl DerefMut for Cqe32 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cqe
    }
}

/// CqeMixed
#[derive(Debug, Default)]
#[repr(C)]
pub struct CqeMix {
    pub cqe: IoUringCqe,
    extra_data: PhantomData<[u64; 2]>,
}

impl CompletionEntry for CqeMix {
    const TYPE: Ty = Ty::CqeMix;
}

impl CqeMix {
    #[inline]
    pub const fn is_cqe32(&self) -> bool {
        self.cqe.flags.contains(IoUringCqeFlags::CQE_32)
    }

    pub const unsafe fn extra_data(&self) -> &[u64; 2] {
        transmute(&self.extra_data)
    }
}

impl Deref for CqeMix {
    type Target = IoUringCqe;

    fn deref(&self) -> &Self::Target {
        &self.cqe
    }
}

impl DerefMut for CqeMix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cqe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cqe_size() {
        assert_eq!(Cqe16::SETUP_CQE_SIZE, 16);
        assert_eq!(Cqe32::SETUP_CQE_SIZE, 32);
        assert_eq!(CqeMix::SETUP_CQE_SIZE, 16);
    }

    #[test]
    fn test_entry_size() {
        assert_eq!(size_of::<Cqe16>(), 16);
        assert_eq!(size_of::<Cqe32>(), 32);
        assert_eq!(size_of::<CqeMix>(), 16);
    }
}
