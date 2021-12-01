use futures::Future;
use serde_json::{map::Map, Value};
use crate::indy::anoncreds;

use crate::error::prelude::*;
use crate::settings;
use crate::utils::constants::*;
use crate::utils::libindy::{
    anoncreds::blob_storage::BlobStorage,
    anoncreds::types::{CredentialInfo, Credential},
    wallet::get_wallet_handle,
    ledger::query::Query,
};

pub struct Holder {}

impl Holder {
    pub fn create_master_secret(master_secret_id: &str) -> VcxResult<String> {
        if settings::indy_mocks_enabled() {
            return Ok(settings::DEFAULT_LINK_SECRET_ALIAS.to_string());
        }

        anoncreds::prover_create_master_secret(get_wallet_handle(),
                                               Some(master_secret_id))
            .wait()
            .map_err(VcxError::from)
    }

    pub fn create_credential_req(prover_did: &str,
                                 credential_offer_json: &str,
                                 credential_def_json: &str) -> VcxResult<(String, String)> {
        if settings::indy_mocks_enabled() {
            return Ok((crate::utils::constants::CREDENTIAL_REQ_STRING.to_owned(), String::new()));
        }

        let master_secret_name = settings::DEFAULT_LINK_SECRET_ALIAS;
        anoncreds::prover_create_credential_req(get_wallet_handle(),
                                                prover_did,
                                                credential_offer_json,
                                                credential_def_json,
                                                master_secret_name)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn get_credentials_for_proof_req(proof_req: &str) -> VcxResult<String> {
        let wallet_handle = get_wallet_handle();

        // this may be too redundant since Prover::search_credentials will validate the proof reqeuest already.
        let proof_request_json: Map<String, Value> = serde_json::from_str(proof_req)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofRequest, format!("Cannot deserialize ProofRequest: {:?}", err)))?;

        // since the search_credentials_for_proof request validates that the proof_req is properly structured, this get()
        // fn should never fail, unless libindy changes their formats.
        let requested_attributes: Option<Map<String, Value>> = proof_request_json.get(REQUESTED_ATTRIBUTES)
            .and_then(|v| {
                serde_json::from_value(v.clone()).map_err(|_| {
                    error!("Invalid Json Parsing of Requested Attributes Retrieved From Libindy. Did Libindy change its structure?");
                }).ok()
            });

        let requested_predicates: Option<Map<String, Value>> = proof_request_json.get(PROOF_REQUESTED_PREDICATES).and_then(|v| {
            serde_json::from_value(v.clone()).map_err(|_| {
                error!("Invalid Json Parsing of Requested Predicates Retrieved From Libindy. Did Libindy change its structure?");
            }).ok()
        });

        // handle special case of "empty because json is bad" vs "empty because no attributes sepected"
        if requested_attributes == None && requested_predicates == None {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidProofRequest, "Proof Request neither contains `requested_attributes` nor `requested_predicates`"));
        }

        let mut fetch_attrs: Map<String, Value> = match requested_attributes {
            Some(attrs) => attrs.clone(),
            None => Map::new()
        };
        match requested_predicates {
            Some(attrs) => fetch_attrs.extend(attrs),
            None => ()
        }
        if 0 < fetch_attrs.len() {
            let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, proof_req, None)
                .wait()
                .map_err(|ec| {
                    error!("Opening Indy Search for Credentials Failed");
                    ec
                })?;
            let creds: String = Self::fetch_credentials(search_handle, fetch_attrs)?;

            // should an error on closing a search handle throw an error, or just a warning?
            // for now we're are just outputting to the user that there is an issue, and continuing on.
            let _ = Self::close_search(search_handle);
            Ok(creds)
        } else {
            Ok("{}".to_string())
        }
    }

    pub fn create_proof(proof_req_json: &str,
                        requested_credentials_json: &str,
                        master_secret_id: &str,
                        schemas_json: &str,
                        credential_defs_json: &str,
                        revoc_states_json: Option<&str>) -> VcxResult<String> {
        if settings::indy_mocks_enabled() {
            return Ok(crate::utils::constants::PROOF_JSON.to_owned());
        }

        anoncreds::prover_create_proof(get_wallet_handle(),
                                       proof_req_json,
                                       requested_credentials_json,
                                       master_secret_id,
                                       schemas_json,
                                       credential_defs_json,
                                       revoc_states_json.unwrap_or("{}"))
            .wait()
            .map_err(VcxError::from)
    }

    fn fetch_credentials(search_handle: i32, requested_attributes: Map<String, Value>) -> VcxResult<String> {
        let mut v: Value = json!({});
        for item_referent in requested_attributes.keys().into_iter() {
            v[ATTRS][item_referent] =
                serde_json::from_str(&anoncreds::prover_fetch_credentials_for_proof_req(search_handle, item_referent, 100).wait()?)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                                      format!("Cannot parse object from JSON string. Err: {:?}", err)))?
        }

        Ok(v.to_string())
    }

    fn close_search(search_handle: i32) -> VcxResult<()> {
        anoncreds::prover_close_credentials_search_for_proof_req(search_handle)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn create_revocation_state(rev_reg_def_json: &str, rev_reg_delta_json: &str, cred_rev_id: &str, tails_file: &str) -> VcxResult<String> {
        if settings::indy_mocks_enabled() {
            return Ok(REV_STATE_JSON.to_string());
        }

        let blob_handle = BlobStorage::open_reader(tails_file)?;

        anoncreds::create_revocation_state(blob_handle, rev_reg_def_json, rev_reg_delta_json, 100, cred_rev_id)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn update_revocation_state(rev_reg_def_json: &str, rev_state_json: &str, rev_reg_delta_json: &str, cred_rev_id: &str, tails_file: &str) -> VcxResult<String> {
        if settings::indy_mocks_enabled() {
            return Ok(REV_STATE_JSON.to_string());
        }

        let blob_handle = BlobStorage::open_reader(tails_file)?;

        anoncreds::update_revocation_state(blob_handle, rev_state_json, rev_reg_def_json, rev_reg_delta_json, 100, cred_rev_id)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn store_credential(cred_id: Option<&str>,
                            cred_req_meta: &str,
                            cred_json: &str,
                            cred_def_json: &str) -> VcxResult<String> {
        if settings::indy_mocks_enabled() {
            return Ok("cred_id".to_string());
        }

        let credential: Credential = serde_json::from_str(&cred_json)
            .map_err(|err| VcxError::from_msg(
                VcxErrorKind::InvalidCredential,
                format!("Cannot parse Credential message from JSON string. Err: {:?}", err),
            ))?;

        let rev_reg_def_json = match credential.rev_reg_id {
            Some(rev_reg_id) => {
                let (_, rev_reg_def_json) = Query::get_rev_reg_def(&rev_reg_id)?;
                Some(rev_reg_def_json)
            }
            None => None
        };

        anoncreds::prover_store_credential(get_wallet_handle(),
                                           cred_id,
                                           cred_req_meta,
                                           cred_json,
                                           cred_def_json,
                                           rev_reg_def_json.as_ref().map(String::as_str))
            .wait()
            .map_err(VcxError::from)
    }

    pub fn delete_credential(cred_id: &str) -> VcxResult<()> {
        if settings::indy_mocks_enabled() { return Ok(()); }

        anoncreds::prover_delete_credential(get_wallet_handle(),
                                            cred_id)
            .wait()
            .map_err(VcxError::from)
    }

    pub fn get_credential(cred_id: &str) -> VcxResult<CredentialInfo> {
        trace!("prover_get_credential >>>");

        let wallet = get_wallet_handle();
        let credential_json = anoncreds::prover_get_credential(wallet, cred_id).wait()?;
        let credential: CredentialInfo = serde_json::from_str(&credential_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                              format!("Can not deserialize the list Credential: {:?}", err))
            )?;

        trace!("prover_get_credential <<< credential: {:?}", secret!(credential));

        Ok(credential)
    }

    pub fn get_credentials() -> VcxResult<Vec<CredentialInfo>> {
        trace!("prover_get_credentials >>>");

        if settings::indy_mocks_enabled() {
            return Ok(Vec::new());
        }

        let wallet = get_wallet_handle();
        let credentials_json = anoncreds::prover_get_credentials(wallet, None).wait()?;
        let credentials: Vec<CredentialInfo> = serde_json::from_str(&credentials_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                              format!("Can not deserialize the list Credentials: {:?}", err))
            )?;

        trace!("prover_get_credentials <<< credentials: {:?}", secret!(credentials));

        Ok(credentials)
    }

    pub fn fetch_public_entities() -> VcxResult<()> {
        trace!("fetch_public_entities >>>");

        let credentials: Vec<CredentialInfo> = Self::get_credentials()?;
        for credential in credentials {
            Query::get_schema(&credential.schema_id)?;
            Query::get_cred_def(&credential.cred_def_id)?;
            if let Some(rev_reg_id) = credential.rev_reg_id.as_ref() {
                Query::get_rev_reg_def(&rev_reg_id)?;
            }
        }

        trace!("fetch_public_entities <<<");
        Ok(())
    }
}