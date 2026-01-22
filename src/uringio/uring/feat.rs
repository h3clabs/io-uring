use std::io::{Error, ErrorKind};

use crate::{
    platform::iouring::IoUringFeatureFlags,
    shared::{
        error::Result,
        null::{Null, NULL},
    },
};

pub fn check_setup_features(features: IoUringFeatureFlags) -> Result<Null> {
    if !features.contains(IoUringFeatureFlags::NODROP) {
        return Err(Error::new(ErrorKind::Other, "Require feature IORING_FEAT_NODROP"));
    }

    Ok(NULL)
}
