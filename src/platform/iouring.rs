pub use rustix::{
    fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd, RawFd},
    io_uring::{
        io_uring_cqe as IoUringCqe, io_uring_params as IoUringParams, io_uring_ptr as IoUringPtr,
        io_uring_setup, io_uring_sqe as IoUringSqe, io_uring_user_data as IoUringUserData,
        IoringCqeFlags as IoUringCqeFlags, IoringFeatureFlags as IoUringFeatureFlags,
        IoringOp as IoUringOp, IoringSetupFlags as IoUringSetupFlags,
        IoringSqeFlags as IoUringSqeFlags, IORING_OFF_CQ_RING as IOURING_OFF_CQ_RING,
        IORING_OFF_SQES as IOURING_OFF_SQES, IORING_OFF_SQ_RING as IOURING_OFF_SQ_RING,
    },
};

// TODO: patch to rustix
#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct IoUringPiAttr {
    pub ptr: IoUringPtr,
    pub mask: u64, // TODO: IORING_RW_ATTR_FLAG_PI (1U << 0)
}

#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct IoUringWritePi {
    pub flags: u16,
    pub app_tag: u16,
    pub len: u32,
    pub addr: u64,
    pub seed: u64,
    pub rsvd: u64,
}
