use futures::Future;
use serde_json::Value;

use std::fmt;
use std::collections::HashMap;

use crate::utils::libindy::wallet::get_wallet_handle;
use crate::utils::libindy::ledger::{libindy_submit_request, libindy_sign_and_submit_request, libindy_sign_request, append_txn_author_agreement_to_request, auth_rule};
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

impl PaymentTxn {
    pub fn from_parts(inputs: Vec<String>, outputs: Vec<Output>, amount: u64, credit: bool) -> PaymentTxn {
        PaymentTxn {
            amount,
            credit,
            inputs,
            outputs,
        }
    }
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

pub fn get_address_info(address: &str) -> VcxResult<AddressInfo> {
    if settings::indy_mocks_enabled() {
        let utxos = json!(
            [
                {
                    "source": build_test_address("1"),
                    "paymentAddress": build_test_address("zR3GN9lfbCVtHjp"),
                    "amount": 1,
                    "extra": "yqeiv5SisTeUGkw"
                },
                {
                    "source": build_test_address("2"),
                    "paymentAddress": build_test_address("zR3GN9lfbCVtHjp"),
                    "amount": 2,
                    "extra": "Lu1pdm7BuAN2WNi"
                }
            ]
        );

        let utxo: Vec<UTXO> = ::serde_json::from_value(utxos).unwrap();

        return Ok(AddressInfo { address: address.to_string(), balance: _address_balance(&utxo), utxo });
    }
    unimplemented!();
}

pub fn list_addresses() -> VcxResult<Vec<String>> {
    if settings::indy_mocks_enabled() {
        let addresses = json!([
                build_test_address("9UFgyjuJxi1i1HD"),
                build_test_address("zR3GN9lfbCVtHjp")
        ]);
        return Ok(::serde_json::from_value(addresses).unwrap());
    }
    unimplemented!();
}

fn is_valid_address(address: &str, method: &str) -> bool {
    static PAY: &str = "pay:";
    address.starts_with(PAY) && address[PAY.len()..].starts_with(method)
}

pub fn get_wallet_token_info() -> VcxResult<WalletInfo> {
    unimplemented!();
}

pub fn get_ledger_fees() -> VcxResult<String> {
    debug!("Ledger: Getting ledger fees");

    if settings::indy_mocks_enabled() { return Ok(DEFAULT_FEES.to_string()); }
    unimplemented!();
}

pub fn send_transaction(req: &str, txn_action: (&str, &str, &str, Option<&str>, Option<&str>)) -> VcxResult<(Option<PaymentTxn>, String)> {
    debug!("send_transaction(req: {}, txn_action: {:?})", secret!(req), secret!(txn_action));

    if settings::indy_mocks_enabled() {
        let inputs = vec!["pay:null:9UFgyjuJxi1i1HD".to_string()];
        let outputs = serde_json::from_str::<Vec<crate::utils::libindy::payments::Output>>(r#"[{"amount":1,"extra":null,"recipient":"pay:null:xkIsxem0YNtHrRO"}]"#).unwrap();
        return Ok((Some(PaymentTxn::from_parts(inputs, outputs, 1, false)), SUBMIT_SCHEMA_RESPONSE.to_string()));
    }
    if settings::get_payment_method().is_err(){
        debug!("Payment Method is not set in the library config. No Payment expected to perform the transaction. Send transactions as is.");
        let txn_response = _submit_request(req)?;
        return Ok((None, txn_response))
    }

    debug!("Payment is not required to perform transaction. Send transactions as is.");
    let txn_response = _submit_request(req)?;
    Ok((None, txn_response))
}

fn _serialize_inputs_and_outputs(inputs: &[String], outputs: &[Output]) -> VcxResult<(String, String)> {
    let inputs = ::serde_json::to_string(inputs)
        .to_vcx(VcxErrorKind::SerializationError, "Cannot serialize inputs")?;
    let outputs = ::serde_json::to_string(outputs)
        .to_vcx(VcxErrorKind::SerializationError, "Cannot serialize outputs")?;
    Ok((inputs, outputs))
}

fn _submit_request(req: &str) -> VcxResult<String> {
    let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    libindy_sign_and_submit_request(&did, req)
}

fn _submit_request_with_fees(req: &str, inputs: &[String], outputs: &[Output]) -> VcxResult<(String, String)> {
    unimplemented!();
}

pub fn pay_a_payee(price: u64, address: &str) -> VcxResult<(PaymentTxn, String)> {
    unimplemented!();
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RequestInfo {
    pub price: u64,
    pub requirements: Vec<::serde_json::Value>
}

fn get_request_info(get_auth_rule_resp_json: &str, requester_info_json: &str, fees_json: &str) -> VcxResult<RequestInfo> {
    unimplemented!();
}

pub fn get_request_price(action_json: String, requester_info_json: Option<String>) -> VcxResult<u64> {
    unimplemented!();
}

fn get_action_price(action: (&str, &str, &str, Option<&str>, Option<&str>), requester_info_json: Option<String>) -> VcxResult<u64> {
    unimplemented!();
}

fn get_requester_info(requester_info_json: Option<String>) -> VcxResult<String> {
    unimplemented!();
}

fn _address_balance(address: &[UTXO]) -> u64 {
    unimplemented!();
}

pub fn inputs(cost: u64) -> VcxResult<(u64, Vec<String>, String)> {
    unimplemented!();
}

pub fn outputs(remainder: u64, refund_address: &str, payee_address: Option<String>, payee_amount: Option<u64>) -> VcxResult<Vec<Output>> {
    unimplemented!();
}

// This is used for testing purposes only!!!
pub fn mint_tokens_and_set_fees(number_of_addresses: Option<u32>, tokens_per_address: Option<u64>, fees: Option<String>, seed: Option<String>) -> VcxResult<()> {
    unimplemented!();
}

pub fn add_new_did(role: Option<&str>) -> (String, String) {
    use crate::indy::ledger;

    let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

    let (did, verkey) = crate::utils::libindy::signus::create_and_store_my_did(None, None).unwrap();
    let mut req_nym = ledger::build_nym_request(&institution_did, &did, Some(&verkey), None, role).wait().unwrap();

    req_nym = append_txn_author_agreement_to_request(&req_nym).unwrap();

    crate::utils::libindy::ledger::libindy_sign_and_submit_request(&institution_did, &req_nym).unwrap();
    (did, verkey)
}
