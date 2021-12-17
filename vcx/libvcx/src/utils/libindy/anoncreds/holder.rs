use futures::Future;
use serde_json::Value;
use std::collections::HashMap;

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
use crate::utils::libindy::anoncreds::types::*;
use crate::utils::libindy::cache::*;
use crate::settings::protocol::ProtocolTypes;
use crate::utils::libindy::anoncreds::{
    utils::attr_common_view,
    proof_request::ProofRequest,
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

    pub fn get_credentials_for_proof_req(proof_request_json: &str) -> VcxResult<String> {
        let wallet_handle = get_wallet_handle();

        // this may be too redundant since Prover::search_credentials will validate the proof reqeuest already.
        let proof_request: ProofRequest = serde_json::from_str(proof_request_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofRequest,
                                              format!("Cannot parse Proof Request from JSON. Err: {:?}", err)))?;

        // handle special case of "empty because json is bad" vs "empty because no attributes sepected"
        if proof_request.requested_attributes.is_empty() && proof_request.requested_predicates.is_empty() {
            return Err(VcxError::from_msg(VcxErrorKind::InvalidProofRequest,
                                          "Proof Request neither contains `requested_attributes` nor `requested_predicates`"));
        }

        let search_handle = anoncreds::prover_search_credentials_for_proof_req(wallet_handle, proof_request_json, None)
            .wait()
            .map_err(|ec| {
                error!("Opening Indy Search for Credentials Failed");
                ec
            })?;

        let credentials_for_proof_request =
            match settings::get_protocol_type() {
                ProtocolTypes::V1 | ProtocolTypes::V2 | ProtocolTypes::V3 => {
                    let mut credentials_for_proof_request = CredentialsForProofRequestV1::new();

                    for (item_referent, requested_attribute) in proof_request.requested_attributes.into_iter() {
                        let credentials = Self::fetch_credentials_for_requested_item(search_handle,
                                                                                     &item_referent,
                                                                                     requested_attribute.name.as_deref(),
                                                                                     requested_attribute.names.as_ref())?;
                        credentials_for_proof_request.attrs.insert(item_referent.to_string(), credentials);
                    }

                    for (item_referent, requested_predicate) in proof_request.requested_predicates.into_iter() {
                        let credentials = Self::fetch_credentials_for_requested_item(search_handle,
                                                                                     &item_referent,
                                                                                     Some(&requested_predicate.name),
                                                                                     None)?;
                        credentials_for_proof_request.attrs.insert(item_referent.to_string(), credentials);
                    }
                    CredentialsForProofRequest::V1(credentials_for_proof_request)
                }
                ProtocolTypes::V4 => {
                    let mut credentials_for_proof_request = CredentialsForProofRequestV2::new();

                    for (item_referent, requested_attribute) in proof_request.requested_attributes.into_iter() {
                        let credentials = Self::fetch_credentials_for_requested_item(search_handle,
                                                                                     &item_referent,
                                                                                     requested_attribute.name.as_deref(),
                                                                                     requested_attribute.names.as_ref())?;
                        let self_attest_allowed = requested_attribute.self_attest_allowed();
                        let missing = credentials.is_empty() && !self_attest_allowed;
                        let credentials_for_item = CredentialsForProofRequestV2Attribute {
                            name: requested_attribute.name,
                            names: requested_attribute.names,
                            credentials,
                            missing,
                            self_attest_allowed,
                        };
                        credentials_for_proof_request.attributes.insert(item_referent, credentials_for_item);
                    }

                    for (item_referent, requested_predicate) in proof_request.requested_predicates.into_iter() {
                        let credentials = Self::fetch_credentials_for_requested_item(search_handle,
                                                                                     &item_referent,
                                                                                     Some(&requested_predicate.name),
                                                                                     None)?;
                        let missing = credentials.is_empty();
                        let credentials_for_item = CredentialsForProofRequestV2Predicate {
                            name: requested_predicate.name,
                            p_type: requested_predicate.p_type,
                            p_value: requested_predicate.p_value,
                            credentials,
                            missing,
                        };
                        credentials_for_proof_request.predicates.insert(item_referent, credentials_for_item);
                    }
                    CredentialsForProofRequest::V2(credentials_for_proof_request)
                }
            };

        Self::close_search(search_handle).ok();

        let credentials_for_proof_request_json = json!(credentials_for_proof_request).to_string();
        Ok(credentials_for_proof_request_json)
    }

    fn fetch_credentials_for_requested_item(search_handle: i32, referent: &str, name: Option<&str>, names: Option<&Vec<String>>) -> VcxResult<Vec<SelectedCredentialInfoWithValue>> {
        let credentials = anoncreds::prover_fetch_credentials_for_proof_req(search_handle, referent, 100).wait()?;
        let credentials: Vec<SelectedCredentialInfo> = serde_json::from_str(&credentials)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                              format!("Cannot parse object from JSON string. Err: {:?}", err)))?;

        let credentials: Vec<SelectedCredentialInfoWithValue> =
            credentials
                .into_iter()
                .flat_map(|credential: SelectedCredentialInfo| {
                    if let Some(name) = name {
                        for (key, val) in credential.cred_info.attrs.iter() {
                            if attr_common_view(&key) == attr_common_view(name) {
                                return Some(SelectedCredentialInfoWithValue {
                                    requested_attributes: map!(name.to_string() => val.to_string()),
                                    cred_info: credential.cred_info,
                                    interval: credential.interval,
                                });
                            }
                        }
                    }
                    if let Some(names) = names {
                        let mut values: HashMap<String, String> = HashMap::new();
                        for name in names {
                            for (key, val) in credential.cred_info.attrs.iter() {
                                if attr_common_view(&key) == attr_common_view(name) {
                                    values.insert(name.to_string(), val.to_string());
                                }
                            }
                        }
                        return Some(SelectedCredentialInfoWithValue {
                            requested_attributes: values,
                            cred_info: credential.cred_info,
                            interval: credential.interval,
                        });
                    }
                    return None;
                })
                .collect();

        Ok(credentials)
    }

    pub fn generate_proof(credentials: &str, self_attested_attrs: &str, proof_req_data_json: &str) -> VcxResult<String> {
        trace!("generate_indy_proof >>> credentials: {}, self_attested_attrs: {}, proof_req_data_json: {}",
               secret!(&credentials), secret!(&self_attested_attrs), secret!(&proof_req_data_json));

        let proof_request: ProofRequest = serde_json::from_str(&proof_req_data_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofRequest,
                                              format!("Cannot parse ProofRequest from JSON string. Err: {}", err)))?;

        let mut selected_credentials = Self::map_selected_credentials(credentials, &proof_request)?;

        let revoc_states = Self::build_rev_states_json(&mut selected_credentials)?;
        let requested_credentials = Self::build_requested_credentials_json(&selected_credentials,
                                                                           self_attested_attrs,
                                                                           &proof_request)?;

        let schemas_json = Self::build_schemas_json(&selected_credentials)?;
        let credential_defs_json = Self::build_cred_def_json(&selected_credentials)?;

        let proof = Self::create_proof(&proof_req_data_json,
                                       &requested_credentials,
                                       settings::DEFAULT_LINK_SECRET_ALIAS,
                                       &schemas_json,
                                       &credential_defs_json,
                                       Some(&revoc_states))?;

        Ok(proof)
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

    pub fn map_selected_credentials(credentials: &str, proof_req: &ProofRequest) -> VcxResult<Vec<ExtendedCredentialInfo>> {
        trace!("credential_def_identifiers >>> credentials: {:?}, proof_req: {:?}", secret!(credentials), secret!(proof_req));
        debug!("Building credential identifiers for proof request");

        let mut identifiers = Vec::new();

        let credentials_selected_for_proof_request: CredentialsSelectedForProofRequest = serde_json::from_str(credentials)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidProofCredentialData,
                                              format!("Cannot parse SelectedCredentials from JSON string. Err: {}", err)))?;

        for (requested_attr, selected_credential) in credentials_selected_for_proof_request.attrs {
            let revocation_interval = proof_req.get_revocation_interval(&requested_attr)?;
            identifiers.push(
                ExtendedCredentialInfo {
                    requested_attr,
                    referent: selected_credential.credential.cred_info.referent,
                    schema_id: selected_credential.credential.cred_info.schema_id,
                    cred_def_id: selected_credential.credential.cred_info.cred_def_id,
                    revocation_interval,
                    timestamp: None,
                    rev_reg_id: selected_credential.credential.cred_info.rev_reg_id,
                    cred_rev_id: selected_credential.credential.cred_info.cred_rev_id,
                    tails_file: selected_credential.tails_file,
                }
            );
        }

        trace!("credential_def_identifiers >>> identifiers: {:?}", secret!(identifiers));
        Ok(identifiers)
    }

    pub fn build_requested_credentials_json(credentials_identifiers: &[ExtendedCredentialInfo],
                                            self_attested_attrs: &str,
                                            proof_req: &ProofRequest) -> VcxResult<String> {
        trace!("build_requested_credentials_json >>> credentials_identifiers: {:?}, self_attested_attrs: {:?}, proof_req: {:?}",
               secret!(credentials_identifiers), secret!(self_attested_attrs), secret!(proof_req));
        debug!("Preparing requested credentials for proof generation");

        let mut requested_credentials = RequestedCredentials::new();

        for ref cred_info in credentials_identifiers {
            if let Some(_) = proof_req.requested_attributes.get(&cred_info.requested_attr) {
                let requested_attribute = RequestedAttribute {
                    cred_id: cred_info.referent.to_owned(),
                    timestamp: cred_info.timestamp,
                    revealed: true,
                };
                requested_credentials.requested_attributes.insert(cred_info.requested_attr.to_owned(), requested_attribute);
            }
        }

        for ref cred_info in credentials_identifiers {
            if let Some(_) = proof_req.requested_predicates.get(&cred_info.requested_attr) {
                let requested_attribute = ProvingCredentialKey {
                    cred_id: cred_info.referent.to_owned(),
                    timestamp: cred_info.timestamp,
                };
                requested_credentials.requested_predicates.insert(cred_info.requested_attr.to_owned(), requested_attribute);
            }
        }

        let self_attested_attributes: HashMap<String, String> = serde_json::from_str(self_attested_attrs)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                              format!("Cannot parse self attested attributes from `self_attested_attrs` JSON string. Err: {}", err)))?;
        requested_credentials.self_attested_attributes = self_attested_attributes;

        let requested_credentials = json!(requested_credentials).to_string();
        trace!("build_requested_credentials_json >>> requested_credentials: {:?}", secret!(requested_credentials));
        Ok(requested_credentials)
    }

    pub fn build_schemas_json(credentials_identifiers: &[ExtendedCredentialInfo]) -> VcxResult<String> {
        trace!("build_schemas_json >>> credentials_identifiers: {:?}", secret!(credentials_identifiers));
        debug!("Getting schemas for proof generation");

        let mut schemas: HashMap<String, Value> = HashMap::new();

        for ref cred_info in credentials_identifiers {
            if schemas.get(&cred_info.schema_id).is_none() {
                let (_, schema_json) = Query::get_schema(&cred_info.schema_id)?;
                let schema_json = serde_json::from_str(&schema_json)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidSchema,
                                                      format!("Cannot parse Schema from Ledger response. Err: {}", err)))?;
                schemas.insert(cred_info.schema_id.to_owned(), schema_json);
            }
        }

        let schemas = json!(schemas).to_string();
        trace!("build_schemas_json <<< schemas: {:?}", secret!(schemas));
        Ok(schemas)
    }

    pub fn build_cred_def_json(credentials_identifiers: &[ExtendedCredentialInfo]) -> VcxResult<String> {
        trace!("build_cred_def_json >>> credentials_identifiers: {:?}", secret!(credentials_identifiers));
        debug!("Getting credential definitions for proof generation");

        let mut cred_defs: HashMap<String, Value> = HashMap::new();

        for ref cred_info in credentials_identifiers {
            if cred_defs.get(&cred_info.cred_def_id).is_none() {
                let (_, credential_def) = Query::get_cred_def(&cred_info.cred_def_id)?;
                let credential_def = serde_json::from_str(&credential_def)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::CredentialDefinitionNotFound,
                                                      format!("Cannot parse Credential Definition from Ledger response. Err: {}", err)))?;
                cred_defs.insert(cred_info.cred_def_id.to_owned(), credential_def);
            }
        }

        let cred_defs = json!(cred_defs).to_string();
        trace!("build_cred_def_json <<< cred_defs: {:?}", secret!(cred_defs));
        Ok(cred_defs)
    }

    // Also updates timestamp in credentials_identifiers
    pub fn build_rev_states_json(credentials_identifiers: &mut Vec<ExtendedCredentialInfo>) -> VcxResult<String> {
        trace!("build_rev_states_json >>> credentials_identifiers: {:?}", secret!(credentials_identifiers));
        debug!("DisclosedProof: Building revocation states");

        let mut rtn: Value = json!({});
        let mut timestamps: HashMap<String, u64> = HashMap::new();

        for cred_info in credentials_identifiers.iter_mut() {
            if let (Some(rev_reg_id), Some(cred_rev_id), Some(tails_file)) =
            (&cred_info.rev_reg_id, &cred_info.cred_rev_id, &cred_info.tails_file) {
                if rtn.get(&rev_reg_id).is_none() {
                    let (from, to) = if let Some(ref interval) = cred_info.revocation_interval
                    { (interval.from, interval.to) } else { (None, None) };

                    //                let from = from.unwrap_or(0);
                    //                let to = to.unwrap_or(time::get_time().sec as u64);
                    let cache = get_rev_reg_cache(&rev_reg_id);

                    let (rev_state_json, timestamp) = if let Some(cached_rev_state) = cache.rev_state {
                        if cached_rev_state.timestamp >= from.unwrap_or(0)
                            && cached_rev_state.timestamp <= to.unwrap_or(time::get_time().sec as u64) {
                            (cached_rev_state.value, cached_rev_state.timestamp)
                        } else {
                            let from = match from {
                                Some(from) if from >= cached_rev_state.timestamp => {
                                    Some(cached_rev_state.timestamp)
                                }
                                _ => None
                            };

                            let (_, rev_reg_def_json) = Query::get_rev_reg_def(&rev_reg_id)?;

                            let (rev_reg_id, rev_reg_delta_json, timestamp) = Query::get_rev_reg_delta(
                                &rev_reg_id,
                                from,
                                to,
                            )?;

                            let rev_state_json = Holder::update_revocation_state(
                                &rev_reg_def_json,
                                &cached_rev_state.value,
                                &rev_reg_delta_json,
                                &cred_rev_id,
                                &tails_file,
                            )?;

                            if timestamp > cached_rev_state.timestamp {
                                let new_cache = RevRegCache {
                                    rev_state: Some(RevState {
                                        timestamp,
                                        value: rev_state_json.clone(),
                                    })
                                };
                                set_rev_reg_cache(&rev_reg_id, &new_cache);
                            }

                            (rev_state_json, timestamp)
                        }
                    } else {
                        let (_, rev_reg_def_json) = Query::get_rev_reg_def(&rev_reg_id)?;

                        let (rev_reg_id, rev_reg_delta_json, timestamp) = Query::get_rev_reg_delta(
                            &rev_reg_id,
                            None,
                            to,
                        )?;

                        let rev_state_json = Holder::create_revocation_state(
                            &rev_reg_def_json,
                            &rev_reg_delta_json,
                            &cred_rev_id,
                            &tails_file,
                        )?;

                        let new_cache = RevRegCache {
                            rev_state: Some(RevState {
                                timestamp,
                                value: rev_state_json.clone(),
                            })
                        };
                        set_rev_reg_cache(&rev_reg_id, &new_cache);

                        (rev_state_json, timestamp)
                    };

                    let rev_state_json: Value = serde_json::from_str(&rev_state_json)
                        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse RevocationState from JSON string. Err: {}", err)))?;

                    // TODO: proover should be able to create multiple states of same revocation policy for different timestamps
                    // see ticket IS-1108
                    rtn[rev_reg_id.to_string()] = json!({timestamp.to_string(): rev_state_json});
                    cred_info.timestamp = Some(timestamp);

                    // Cache timestamp for future attributes that have the same rev_reg_id
                    timestamps.insert(rev_reg_id.to_string(), timestamp);
                }

                // If the rev_reg_id is already in the map, timestamp may not be updated on cred_info
                if cred_info.timestamp.is_none() {
                    cred_info.timestamp = timestamps.get(rev_reg_id).cloned();
                }
            }
        }

        trace!("build_rev_states_json <<< states: {:?}", secret!(rtn));

        Ok(rtn.to_string())
    }
}
