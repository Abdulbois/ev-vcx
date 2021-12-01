use serde_json;
use futures::Future;
use std::sync::{mpsc, Arc};
use std::thread;

use super::request::Request;
use super::response::Response;
use crate::indy::vdr;
use crate::settings;
use crate::error::prelude::*;
use crate::utils::constants::*;
use crate::schema::SchemaData;
use crate::utils::libindy::wallet::get_wallet_handle;
use crate::utils::libindy::anoncreds::types::CredentialDefinitionData;
use crate::utils::libindy::vdr::{get_vdr, VDRInfo};
use crate::utils::qualifier::is_fully_qualified;
use crate::indy::vdr::VDR;

pub struct Query {}

impl Query {
    fn get_schema_func(vdr: &VDR, schema_id: String, namespace: Option<String>) -> Option<(String, String)> {
        let wallet_handle = get_wallet_handle();

        let fqschema = match namespace {
            Some(namespace) => format!("schema:{}:did:{}:{}", namespace, namespace, schema_id),
            None => schema_id.to_string()
        };

        println!("resolve_schema_with_cache {}", fqschema);

        if let Ok(schema_json) = vdr::resolve_schema_with_cache(&vdr,
                                                                wallet_handle,
                                                                &fqschema,
                                                                "{}").wait() {
            println!("schema_json {}", schema_json);

            let valid_schema_data = serde_json::from_str::<SchemaData>(&schema_json);
            if valid_schema_data.is_ok() {
                return Some((schema_id.to_string(), schema_json));
            }
        }
        return None;
    }

    pub fn get_schema(schema_id: &str) -> VcxResult<(String, String)> {
        if settings::indy_mocks_enabled() { return Ok((SCHEMA_ID.to_string(), SCHEMA_JSON.to_string())); }
        println!("get_schema {}", schema_id);

        match Self::query_connected_pool_networks(Arc::new(Self::get_schema_func), schema_id)? {
            Some(result) => Ok(result),
            None =>
                Err(VcxError::from_msg(VcxErrorKind::InvalidSchema,
                                       format!("Could not find Schema on the connected Ledger networks")))
        }
    }

    pub fn get_cred_def_func(vdr: &VDR, cred_def_id: String, namespace: Option<String>) -> Option<(String, String)> {
        let wallet_handle = get_wallet_handle();

        let fqcreddef = match namespace {
            Some(namespace) => format!("creddef:{}:did:{}:{}", namespace, namespace, cred_def_id),
            None => cred_def_id.to_string()
        };

        println!("get_cred_def_func {}", fqcreddef);

        if let Ok(cred_def_json) = vdr::resolve_cred_def_with_cache(&vdr,
                                                                    wallet_handle,
                                                                    &fqcreddef,
                                                                    "{}").wait() {
            let valid_cred_def_data = serde_json::from_str::<CredentialDefinitionData>(&cred_def_json);
            if valid_cred_def_data.is_ok() {
                return Some((cred_def_id, cred_def_json));
            }
        }
        return None;
    }

    pub fn get_cred_def(cred_def_id: &str) -> VcxResult<(String, String)> {
        if settings::indy_mocks_enabled() { return Ok((CRED_DEF_ID.to_string(), CRED_DEF_JSON.to_string())); }

        match Self::query_connected_pool_networks(Arc::new(Self::get_cred_def_func), cred_def_id)? {
            Some(result) => Ok(result),
            None =>
                Err(VcxError::from_msg(VcxErrorKind::CredentialDefinitionNotFound,
                                       format!("Could not find Credential Definition on the connected Ledger networks")))
        }
    }

    pub fn get_rev_reg_def(rev_reg_id: &str) -> VcxResult<(String, String)> {
        if settings::indy_mocks_enabled() { return Ok((REV_REG_ID.to_string(), rev_def_json())); }

        let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

        Request::get_revoc_reg_def(&submitter_did, rev_reg_id)
            .and_then(|req| Request::submit(&req))
            .and_then(|response| Response::parse_get_revoc_reg_def_response(&response))
    }

    pub fn get_rev_reg_delta(rev_reg_id: &str, from: Option<u64>, to: Option<u64>)
                             -> VcxResult<(String, String, u64)> {
        if settings::indy_mocks_enabled() { return Ok((REV_REG_ID.to_string(), REV_REG_DELTA_JSON.to_string(), 1)); }

        let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;
        let from: i64 = if let Some(_from) = from { _from as i64 } else { -1 };
        let to = if let Some(_to) = to { _to as i64 } else { time::get_time().sec };

        Request::get_revoc_reg_delta(&submitter_did, rev_reg_id, from, to)
            .and_then(|req| Request::submit(&req))
            .and_then(|response| Response::parse_get_revoc_reg_delta_response(&response))
    }

    pub fn get_rev_reg(rev_reg_id: &str, timestamp: u64) -> VcxResult<(String, String, u64)> {
        if settings::indy_mocks_enabled() {
            return Ok((REV_REG_ID.to_string(), REV_REG_JSON.to_string(), 1));
        }

        let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

        Request::get_revoc_reg(&submitter_did, rev_reg_id, timestamp)
            .and_then(|req| Request::submit(&req))
            .and_then(|response| Response::parse_get_revoc_reg_response(&response))
    }

    pub fn query_connected_pool_networks(
        query_func: Arc<dyn Fn(&VDR, String, Option<String>) -> Option<(String, String)> + Send + Sync>,
        id: &str,
    ) -> VcxResult<Option<(String, String)>> {
        let receiver = {
            let (sender, receiver) = mpsc::channel();

            let vdr: &VDRInfo = get_vdr()?;

            if is_fully_qualified(&id) {
                let sender_ = sender.clone();
                let id = id.to_string();
                let query_func = query_func.clone();

                thread::spawn(move || {
                    sender_.send(query_func(&vdr.vdr, id, None)).ok();
                });
            } else {
                for namespace in &vdr.namespace_list {
                    let sender_ = sender.clone();
                    let id = id.to_string();
                    let namespace = namespace.to_string();
                    let query_func = query_func.clone();

                    thread::spawn(move || {
                        sender_.send(query_func(&vdr.vdr, id, Some(namespace))).ok();
                    });
                }
            }

            receiver
        };

        for received in receiver {
            if received.is_some() {
                return Ok(received);
            }
        }

        return Ok(None);
    }
}


