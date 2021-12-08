use std::collections::HashMap;
use std::sync::Once;
use std::sync::Mutex;
use serde_json;
use futures::Future;

use super::request::Request;
use crate::indy::ledger;
use crate::settings;
use crate::utils::libindy::wallet::get_wallet_handle;
use crate::error::prelude::*;

/**
   Structure for parsing GET_AUTH_RULE response
    # parameters
   result - the payload containing data relevant to the GET_AUTH_RULE transaction
*/
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAuthRuleResponse {
    pub result: GetAuthRuleResult
}

/**
   Structure of the result value within the GAT_AUTH_RULE response
    # parameters
   identifier - The DID this request was submitted from
   req_id - Unique ID number of the request with transaction
   txn_type - the type of transaction that was submitted
   data - A key:value map with the action id as the key and the auth rule as the value
*/
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAuthRuleResult {
    pub identifier: String,
    pub req_id: u64,
    // This is to change the json key to adhear to the functionality on ledger
    #[serde(rename = "type")]
    pub txn_type: String,
    pub data: Vec<AuthRule>,
}

/**
   Enum of the constraint type within the GAT_AUTH_RULE result data
    # parameters
   Role - The final constraint
   And - Combine multiple constraints all of them must be met
   Or - Combine multiple constraints any of them must be met
   Forbidden - action is forbidden
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "constraint_id")]
pub enum Constraint {
    #[serde(rename = "OR")]
    OrConstraint(CombinationConstraint),
    #[serde(rename = "AND")]
    AndConstraint(CombinationConstraint),
    #[serde(rename = "ROLE")]
    RoleConstraint(RoleConstraint),
    #[serde(rename = "FORBIDDEN")]
    ForbiddenConstraint(ForbiddenConstraint),
}

/**
   The final constraint
    # parameters
   sig_count - The number of signatures required to execution action
   role - The role which the user must have to execute the action.
   metadata -  An additional parameters of the constraint (contains transaction FEE cost).
   need_to_be_owner - The flag specifying if a user must be an owner of the transaction.
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoleConstraint {
    pub sig_count: Option<u32>,
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub need_to_be_owner: Option<bool>,
}

/**
   The empty constraint means that action is forbidden
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ForbiddenConstraint {}

/**
   The constraint metadata
    # parameters
   fees - The action cost
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub fees: Option<String>,
}

/**
   Combine multiple constraints
    # parameters
   auth_constraints - The type of the combination
*/
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CombinationConstraint {
    pub auth_constraints: Vec<Constraint>
}

/* Map contains default Auth Rules set on the Ledger*/
lazy_static! {
        static ref AUTH_RULES: Mutex<Vec<AuthRule>> = Default::default();
    }

/* Helper structure to store auth rule set on the Ledger */
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthRule {
    auth_action: String,
    auth_type: String,
    field: String,
    old_value: Option<String>,
    new_value: Option<String>,
    constraint: Constraint,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Action {
    pub auth_type: String,
    pub auth_action: String,
    pub field: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

// Helpers to set fee alias for auth rules
pub fn set_actions_fee_aliases(submitter_did: &str, rules_fee: &str) -> VcxResult<()> {
    _get_default_ledger_auth_rules();

    let auth_rules = AUTH_RULES.lock().unwrap();

    let fees: HashMap<String, String> = ::serde_json::from_str(rules_fee)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot deserialize Fees: {:?}", err)))?;

    let mut auth_rules: Vec<AuthRule> = auth_rules.clone();

    auth_rules
        .iter_mut()
        .for_each(|auth_rule| {
            if let Some(fee_alias) = fees.get(&auth_rule.auth_type) {
                _set_fee_to_constraint(&mut auth_rule.constraint, &fee_alias);
            }
        });

    _send_auth_rules(submitter_did, &auth_rules)
}

fn _send_auth_rules(submitter_did: &str, data: &[AuthRule]) -> VcxResult<()> {
    let data = serde_json::to_string(data)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError,
                                          format!("Cannot serialize auth rules: {:?}", err)))?;

    let auth_rules_request = Request::auth_rules(submitter_did, &data)?;

//        let pool = get_pool()?;

    let response = ledger::sign_and_submit_request(0,
                                                   get_wallet_handle(),
                                                   submitter_did,
                                                   &auth_rules_request)
        .wait()?;

    let response: serde_json::Value = ::serde_json::from_str(&response)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse,
                                          format!("Could not parse Ledger response for SET AUTH_RULES. Err: {:?}", err)))?;

    match response["op"].as_str().unwrap_or_default() {
        "REPLY" => Ok(()),
        _ => Err(VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse,
                                    format!("Could not submit AUTH_RULES transaction on the Ledger. Response: {:?}", response)))?
    }
}

fn _get_default_ledger_auth_rules() {
    lazy_static! {
            static ref GET_DEFAULT_AUTH_CONSTRAINTS: Once = Once::new();

        }

    GET_DEFAULT_AUTH_CONSTRAINTS.call_once(|| {
        let get_auth_rule_request = crate::indy::ledger::build_get_auth_rule_request(None,
                                                                                     None,
                                                                                     None,
                                                                                     None,
                                                                                     None,
                                                                                     None).wait().unwrap();
        let get_auth_rule_response = Request::submit(&get_auth_rule_request).unwrap();

        let response: GetAuthRuleResponse = ::serde_json::from_str(&get_auth_rule_response)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse,
                                              format!("Could not parse Ledger response for GET ALL_AUTH_RULES. Err: {:?}", err))).unwrap();

        let mut auth_rules = AUTH_RULES.lock().unwrap();
        *auth_rules = response.result.data;
    })
}

fn _set_fee_to_constraint(constraint: &mut Constraint, fee_alias: &str) {
    match constraint {
        Constraint::RoleConstraint(constraint) => {
            constraint.metadata.as_mut().map(|meta| meta.fees = Some(fee_alias.to_string()));
        }
        Constraint::AndConstraint(constraint) | Constraint::OrConstraint(constraint) => {
            for mut constraint in constraint.auth_constraints.iter_mut() {
                _set_fee_to_constraint(&mut constraint, fee_alias)
            }
        }
        Constraint::ForbiddenConstraint(_) => {}
    }
}

pub fn get_action_auth_rule(action: (&str, &str, &str, Option<&str>, Option<&str>)) -> VcxResult<String> {
    let (txn_type, action, field, old_value, new_value) = action;

    if settings::indy_mocks_enabled() { return Ok(json!({"result":{"data":[{"new_value":"0","constraint":{"need_to_be_owner":false,"sig_count":1,"metadata":{"fees":txn_type},"role":"0","constraint_id":"ROLE"},"field":"role","auth_type":"1","auth_action":"ADD"}],"identifier":"LibindyDid111111111111","auth_action":"ADD","new_value":"0","reqId":15616,"auth_type":"1","type":"121","field":"role"},"op":"REPLY"}).to_string()); }

    let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    let request = Request::get_auth_rule(Some(&did), Some(txn_type), Some(action), Some(field), old_value, new_value)?;

    let response_json = Request::submit(&request)?;

    let response: serde_json::Value = ::serde_json::from_str(&response_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse,
                                          format!("Could not parse Ledger response for GET_AUTH_RULE. Err: {:?}", err)))?;

    match response["op"].as_str().unwrap_or_default() {
        "REPLY" => Ok(response_json),
        _ => Err(VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse,
                                    format!("Could not get the list of GET_AUTH_RULE set on the Ledger. Response: {:?}", response)))?
    }
}