use futures::Future;
use vdrtools_sys::CommandHandle;

use crate::indy::blob_storage;
use crate::error::prelude::*;

pub struct BlobStorage {}

impl BlobStorage {
    const BLOB_STORAGE_TYPE: &'static str = "default";

    pub fn open_reader(base_dir: &str) -> VcxResult<i32> {
        let tails_config = json!({
            "base_dir": base_dir,
            "uri_pattern": ""
        }).to_string();
        blob_storage::open_reader("default", &tails_config)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn open_writer(base_dir: &str) -> VcxResult<CommandHandle> {
        let tails_config = json!({
            "base_dir": base_dir,
            "uri_pattern": ""
        }).to_string();

        blob_storage::open_writer(Self::BLOB_STORAGE_TYPE, &tails_config)
            .wait()
            .map_err(VcxError::from)
    }
}