use futures::Future;
use crate::indy::anoncreds;
use crate::settings;
use crate::error::prelude::*;
use crate::utils::constants::*;
use crate::utils::libindy::{
    anoncreds::blob_storage::BlobStorage,
    ledger::utils::publish_rev_reg_delta,
    wallet::get_wallet_handle, LibindyMock,
};

pub struct Issuer {}

impl Issuer {
    const REVOCATION_REGISTRY_TYPE: &'static str = "ISSUANCE_BY_DEFAULT";

    pub fn create_schema(issuer_did: &str,
                         name: &str,
                         version: &str,
                         attrs: &str) -> VcxResult<(String, String)> {
        if settings::indy_mocks_enabled() {
            return Ok((SCHEMA_ID.to_string(), SCHEMA_JSON.to_string()));
        }

        anoncreds::issuer_create_schema(issuer_did,
                                        name,
                                        version,
                                        attrs)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn create_and_store_credential_def(issuer_did: &str,
                                           schema_json: &str,
                                           tag: &str,
                                           sig_type: Option<&str>,
                                           support_revocation: Option<bool>,
    ) -> VcxResult<(String, String)> {
        if settings::indy_mocks_enabled() {
            return Ok((CRED_DEF_ID.to_string(), CRED_DEF_JSON.to_string()));
        }

        let config_json = json!({
            "support_revocation": support_revocation.unwrap_or(false)
        }).to_string();

        let wallet_handle = get_wallet_handle();
        anoncreds::issuer_create_and_store_credential_def(wallet_handle,
                                                          issuer_did,
                                                          schema_json,
                                                          tag,
                                                          sig_type,
                                                          &config_json)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn create_and_store_revoc_reg(issuer_did: &str,
                                      cred_def_id: &str,
                                      tails_path: &str,
                                      max_creds: u32) -> VcxResult<(String, String, String)> {
        trace!("creating revocation registry: {}, {}, {}", secret!(cred_def_id), secret!(tails_path), secret!(max_creds));

        if settings::indy_mocks_enabled() { return Ok((REV_REG_ID.to_string(), rev_def_json(), "".to_string())); }

        let wallet_handle = get_wallet_handle();

        let writer = BlobStorage::open_writer(tails_path)?;

        let revoc_config = json!({
            "max_cred_num": max_creds,
            "issuance_type": Self::REVOCATION_REGISTRY_TYPE
        }).to_string();

        anoncreds::issuer_create_and_store_revoc_reg(wallet_handle,
                                                     issuer_did,
                                                     None,
                                                     "tag1",
                                                     cred_def_id,
                                                     &revoc_config,
                                                     writer)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn create_credential_offer(cred_def_id: &str) -> VcxResult<String> {
        if settings::indy_mocks_enabled() {
            let rc = LibindyMock::get_result();
            if rc != 0 { return Err(VcxError::from(VcxErrorKind::InvalidState)); };
            return Ok(LIBINDY_CRED_OFFER.to_string());
        }

        let wallet_handle = get_wallet_handle();

        anoncreds::issuer_create_credential_offer(wallet_handle,
                                                  cred_def_id)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn create_credential(cred_offer_json: &str,
                             cred_req_json: &str,
                             cred_values_json: &str,
                             rev_reg_id: Option<&str>,
                             tails_file: Option<&str>) -> VcxResult<(String, Option<String>, Option<String>)> {
        if settings::indy_mocks_enabled() {
            return Ok((CREDENTIAL_JSON.to_owned(), None, None));
        }

        let wallet_handle = get_wallet_handle();

        let blob_handle = match tails_file {
            Some(tails_file_) => BlobStorage::open_reader(&tails_file_)?,
            None => -1,
        };
        anoncreds::issuer_create_credential(wallet_handle,
                                            cred_offer_json,
                                            cred_req_json,
                                            cred_values_json,
                                            rev_reg_id,
                                            blob_handle)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn revoke_credential(tails_file: &str, rev_reg_id: &str, cred_rev_id: &str) -> VcxResult<String> {
        if settings::indy_mocks_enabled() {
            return Ok(REV_REG_DELTA_JSON.to_string());
        }

        let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

        let blob_handle = BlobStorage::open_reader(tails_file)?;
        let wallet_handle = get_wallet_handle();

        let delta = anoncreds::issuer_revoke_credential(wallet_handle,
                                                        blob_handle,
                                                        rev_reg_id,
                                                        cred_rev_id)
            .wait()
            .map_err(VcxError::from)?;

        publish_rev_reg_delta(&submitter_did, rev_reg_id, &delta)?;

        Ok(delta)
    }
}