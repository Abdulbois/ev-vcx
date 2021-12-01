use futures::Future;

use crate::indy::{ledger, vdr};
use crate::settings;
use crate::utils::libindy::vdr::get_vdr;
use crate::utils::libindy::wallet::get_wallet_handle;
use crate::error::prelude::*;
use crate::utils::constants::*;
use crate::utils::libindy::vdr::DEFAULT_NETWORK;

pub struct Request {}

impl Request {
    pub fn multisign(did: &str, request: &str) -> VcxResult<String> {
        ledger::multi_sign_request(get_wallet_handle(), did, request)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn sign(did: &str, request: &str) -> VcxResult<String> {
        ledger::sign_request(get_wallet_handle(), did, request)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn sign_and_submit(request_json: &str) -> VcxResult<String> {
        if settings::indy_mocks_enabled() { return Ok(r#"{"rc":"success"}"#.to_string()); }

        let vdr = get_vdr()?;
        let wallet_handle = get_wallet_handle();
        let issuer_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

        let signed_request =
            ledger::sign_request(wallet_handle, &issuer_did, request_json)
                .wait()
                .map_err(VcxError::from)?;

        vdr::submit_raw_txn(&vdr.vdr, DEFAULT_NETWORK, &signed_request.as_bytes())
            .wait()
            .map_err(VcxError::from)
    }

    pub fn submit(request_json: &str) -> VcxResult<String> {
        let vdr = get_vdr()?;

        vdr::submit_raw_txn(&vdr.vdr, DEFAULT_NETWORK, &request_json.as_bytes())
            .wait()
            .map_err(VcxError::from)
    }

    pub fn append_txn_author_agreement(request_json: &str) -> VcxResult<String> {
        if let Some(author_agreement) = crate::utils::author_agreement::get_txn_author_agreement()? {
            ledger::append_txn_author_agreement_acceptance_to_request(request_json,
                                                                      author_agreement.text.as_ref().map(String::as_str),
                                                                      author_agreement.version.as_ref().map(String::as_str),
                                                                      author_agreement.taa_digest.as_ref().map(String::as_str),
                                                                      &author_agreement.acceptance_mechanism_type,
                                                                      author_agreement.time_of_acceptance)
                .wait()
                .map_err(VcxError::from)
        } else {
            Ok(request_json.to_string())
        }
    }

    pub fn auth_rules(submitter_did: &str, data: &str) -> VcxResult<String> {
        ledger::build_auth_rules_request(submitter_did, data)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn get_auth_rule(submitter_did: Option<&str>,
                         txn_type: Option<&str>,
                         action: Option<&str>,
                         field: Option<&str>,
                         old_value: Option<&str>,
                         new_value: Option<&str>) -> VcxResult<String> {
        ledger::build_get_auth_rule_request(submitter_did, txn_type, action, field, old_value, new_value)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn get_nym(submitter_did: Option<&str>, did: &str) -> VcxResult<String> {
        ledger::build_get_nym_request(submitter_did, did)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn get_revoc_reg_def(submitter_did: &str,
                             rev_reg_id: &str) -> VcxResult<String> {
        ledger::build_get_revoc_reg_def_request(Some(submitter_did), rev_reg_id)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn get_revoc_reg_delta(submitter_did: &str,
                               rev_reg_id: &str,
                               from: i64,
                               to: i64) -> VcxResult<String> {
        ledger::build_get_revoc_reg_delta_request(Some(submitter_did),
                                                  rev_reg_id,
                                                  from,
                                                  to)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn get_revoc_reg(submitter_did: &str, rev_reg_id: &str, timestamp: u64) -> VcxResult<String> {
        ledger::build_get_revoc_reg_request(Some(submitter_did),
                                            rev_reg_id,
                                            timestamp as i64)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn schema(schema: &str) -> VcxResult<String> {
        if settings::indy_mocks_enabled() {
            return Ok(SCHEMA_TXN.to_string());
        }

        let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

        let request = ledger::build_schema_request(&submitter_did, schema)
            .wait()
            .map_err(VcxError::from)?;

        Request::append_txn_author_agreement(&request)
    }

    pub fn rev_reg(issuer_did: &str, rev_reg_def_json: &str) -> VcxResult<String> {
        if settings::indy_mocks_enabled() { return Ok("".to_string()); }

        let rev_reg_def_req = ledger::build_revoc_reg_def_request(issuer_did, rev_reg_def_json)
            .wait()
            .map_err(VcxError::from)?;

        Request::append_txn_author_agreement(&rev_reg_def_req)
    }

    pub fn rev_reg_delta(issuer_did: &str, rev_reg_id: &str, rev_reg_entry_json: &str)
                         -> VcxResult<String> {
        let request = ledger::build_revoc_reg_entry_request(issuer_did, rev_reg_id, REVOC_REG_TYPE, rev_reg_entry_json)
            .wait()
            .map_err(VcxError::from)?;

        Request::append_txn_author_agreement(&request)
    }

    pub fn credential_definition(issuer_did: &str, cred_def_json: &str) -> VcxResult<String> {
        if settings::indy_mocks_enabled() {
            return Ok(CRED_DEF_REQ.to_string());
        }

        let cred_def_req = ledger::build_cred_def_request(issuer_did, cred_def_json)
            .wait()
            .map_err(VcxError::from)?;

        Request::append_txn_author_agreement(&cred_def_req)
    }

    pub fn set_endorser(request: &str, endorser: &str) -> VcxResult<String> {
        if settings::indy_mocks_enabled() { return Ok(REQUEST_WITH_ENDORSER.to_string()); }

        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

        let request = ledger::append_request_endorser(request, endorser).wait()?;

        Request::multisign(&did, &request)
    }
}