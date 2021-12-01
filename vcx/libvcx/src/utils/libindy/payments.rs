use futures::Future;
use serde_json::Value;

use std::fmt;
use std::collections::HashMap;

use crate::utils::libindy::wallet::get_wallet_handle;
use crate::utils::libindy::ledger::request::Request;
use crate::utils::constants::{SUBMIT_SCHEMA_RESPONSE,};
use crate::settings;
use crate::error::prelude::*;

static DEFAULT_FEES: &str = r#"{"0":0, "1":0, "3":0, "100":0, "101":2, "102":42, "103":0, "104":0, "105":0, "107":0, "108":0, "109":0, "110":0, "111":0, "112":0, "113":2, "114":2, "115":0, "116":0, "117":0, "118":0, "119":0, "10001":0}"#;

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletInfo {
    balance: u64,
    balance_str: String,
    addresses: Vec<AddressInfo>,
}

impl WalletInfo {
    pub fn get_balance(&self) -> u64 {
        self.balance
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AddressInfo {
    pub address: String,
    pub balance: u64,
    utxo: Vec<UTXO>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct UTXO {
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    #[serde(rename = "paymentAddress")]
    recipient: String,
    amount: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct Output {
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<String>,
    recipient: String,
    amount: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra: Option<String>,
}

impl fmt::Display for WalletInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match ::serde_json::to_string(&self) {
            Ok(s) => write!(f, "{}", s),
            Err(_) => write!(f, "null"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PaymentTxn {
    pub amount: u64,
    pub credit: bool,
    pub inputs: Vec<String>,
    pub outputs: Vec<Output>,
}

pub fn build_test_address(address: &str) -> String {
    format!("pay:{}:{}", crate::settings::get_payment_method().unwrap_or_default(), address)
}

pub fn create_address(seed: Option<String>) -> VcxResult<String> {
    trace!("create_address >>> seed: {:?}", secret!(seed));

    if settings::indy_mocks_enabled() {
        return Ok(build_test_address("J81AxU9hVHYFtJc"));
    }

    unimplemented!();
}

pub fn sign_with_address(address: &str, message: &[u8]) -> VcxResult<Vec<u8>> {
    trace!("sign_with_address >>> address: {:?}, message: {:?}", secret!(address), secret!(message));

    if settings::indy_mocks_enabled() {return Ok(Vec::from(message).to_owned()); }

    unimplemented!();
}

pub fn verify_with_address(address: &str, message: &[u8], signature: &[u8]) -> VcxResult<bool> {
    trace!("sign_with_address >>> address: {:?}, message: {:?}", secret!(address), secret!(message));

    if settings::indy_mocks_enabled() { return Ok(true); }
    unimplemented!();
}

pub fn get_wallet_token_info() -> VcxResult<WalletInfo> {
    unimplemented!();
}

pub fn get_ledger_fees() -> VcxResult<String> {
    debug!("Ledger: Getting ledger fees");

    if settings::indy_mocks_enabled() { return Ok(DEFAULT_FEES.to_string()); }
    unimplemented!();
}


pub fn pay_a_payee(price: u64, address: &str) -> VcxResult<(PaymentTxn, String)> {
    unimplemented!();
}

pub fn get_request_price(action_json: String, requester_info_json: Option<String>) -> VcxResult<u64> {
    unimplemented!();
}

fn _address_balance(address: &[UTXO]) -> u64 {
    unimplemented!();
}

pub fn add_new_did(role: Option<&str>) -> (String, String) {
    use crate::indy::ledger;

    let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

    let (did, verkey) = crate::utils::libindy::crypto::create_and_store_my_did(None, None).unwrap();
    let mut req_nym = ledger::build_nym_request(&institution_did, &did, Some(&verkey), None, role).wait().unwrap();

    req_nym = Request::append_txn_author_agreement(&req_nym).unwrap();
    Request::sign_and_submit(&req_nym).unwrap();
    (did, verkey)
}
