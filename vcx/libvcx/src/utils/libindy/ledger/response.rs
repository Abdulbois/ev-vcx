use serde_json;
use futures::Future;

use crate::indy::ledger;
use crate::error::prelude::*;
use crate::utils::libindy::ledger::types::Response as TxnResponse;

pub struct Response {}

impl Response {
    pub fn parse(response: &str) -> VcxResult<TxnResponse> {
        serde_json::from_str::<TxnResponse>(response)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse,
                                              format!("Could not parse Ledger response. Err: {:?}", err)))
    }

    pub fn parse_get_revoc_reg_def_response(rev_reg_def_json: &str) -> VcxResult<(String, String)> {
        ledger::parse_get_revoc_reg_def_response(rev_reg_def_json)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn parse_get_revoc_reg_response(get_rev_reg_resp: &str) -> VcxResult<(String, String, u64)> {
        ledger::parse_get_revoc_reg_response(get_rev_reg_resp)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn parse_get_revoc_reg_delta_response(get_rev_reg_delta_response: &str)
                                              -> VcxResult<(String, String, u64)> {
        ledger::parse_get_revoc_reg_delta_response(get_rev_reg_delta_response)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn check_schema_response(response: &str) -> VcxResult<()> {
        match Self::parse(response)? {
            TxnResponse::Reply(_) => Ok(()),
            TxnResponse::Reject(reject) => Err(VcxError::from_msg(VcxErrorKind::DuplicationSchema, format!("{:?}", reject))),
            TxnResponse::ReqNACK(reqnack) => Err(VcxError::from_msg(VcxErrorKind::UnknownSchemaRejection, format!("{:?}", reqnack)))
        }
    }
}