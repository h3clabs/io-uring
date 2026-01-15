pub mod fs;
pub mod net;
pub mod opcode;

use crate::{
    platform::iouring::IoUringOp,
    uringio::submission::entry::{Sqe128, Sqe64},
};

mod private {
    use super::*;

    /// Sealed Entry: Sqe64 and Sqe128
    pub trait Sealed {}

    impl Sealed for Sqe64 {}
    impl Sealed for Sqe128 {}
}

pub trait Op: Sized {
    type Entry: private::Sealed;

    const OP_CODE: IoUringOp;

    // FIX: Associated constants lazy evaluation, so do check in test
    fn check_size_align() {
        assert_eq!(size_of::<Self>(), size_of::<Self::Entry>());
        assert_eq!(align_of::<Self>(), align_of::<Self::Entry>());
    }
}

#[cfg(feature = "unstable-toolchain")]
mod _unsafe_transmute_ {
    use std::intrinsics::transmute_unchecked;

    use super::*;

    impl<T: Op<Entry = Sqe64>> From<T> for Sqe64 {
        fn from(op: T) -> Self {
            // SAFETY: Op<Entry = Sqe64> size & align checked
            unsafe { transmute_unchecked(op) }
        }
    }

    impl<T: Op<Entry = Sqe128>> From<T> for Sqe128 {
        fn from(op: T) -> Self {
            // SAFETY: Op<Entry = Sqe128> size & align checked
            unsafe { transmute_unchecked(op) }
        }
    }
}

#[cfg(not(feature = "unstable-toolchain"))]
mod _unsafe_transmute_ {
    use std::{mem::ManuallyDrop, ptr};

    use super::*;

    impl<T: Op<Entry = Sqe64>> From<T> for Sqe64 {
        fn from(op: T) -> Self {
            let op = ManuallyDrop::new(op);
            // SAFETY: Op<Entry = Sqe64> size & align checked
            unsafe { ptr::read((&raw const op).cast()) }
        }
    }

    impl<T: Op<Entry = Sqe128>> From<T> for Sqe128 {
        fn from(op: T) -> Self {
            let op = ManuallyDrop::new(op);
            // SAFETY: Op<Entry = Sqe128> size & align checked
            unsafe { ptr::read((&raw const op).cast()) }
        }
    }
}
