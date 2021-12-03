use serde_json;
use futures::Future;

use super::request::Request;
use super::response::Response;
use crate::indy::{ledger, crypto, vdr};
use crate::settings;
use crate::utils::libindy::vdr::{get_vdr, get_namespace, DEFAULT_NETWORK};
use crate::utils::libindy::wallet::get_wallet_handle;
use crate::error::prelude::*;
use crate::utils::libindy::vdr::VDRInfo;
use crate::utils::libindy::ledger::types::Transaction;
use crate::utils::libindy::ledger::types::Response as TransactionResponse;
use crate::utils::qualifier;

pub fn publish_cred_def(cred_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(String::new()); }

    sign_and_submit_txn(cred_def_json, TxnTypes::CredDef)
}

pub fn publish_schema(schema: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(String::new()); }

    let response = sign_and_submit_txn(schema, TxnTypes::Schema)?;
    Response::check_schema_response(&response)?;
    Ok(response)
}

pub fn publish_rev_reg_def(issuer_did: &str, rev_reg_def_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(String::new()); }

    let request_json = Request::rev_reg(issuer_did, &rev_reg_def_json)?;
    sign_and_submit_raw_txn(&request_json)
}


pub fn publish_rev_reg_delta(issuer_did: &str, rev_reg_id: &str, rev_reg_entry_json: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(String::new()); }

    let request_json = Request::rev_reg_delta(issuer_did, rev_reg_id, rev_reg_entry_json)?;
    sign_and_submit_raw_txn(&request_json)
}

pub enum TxnTypes {
    DID,
    Schema,
    CredDef,
}

pub fn sign_and_submit_txn(txn_data: &str, txn_type: TxnTypes) -> VcxResult<String> {
    let vdr: &VDRInfo = get_vdr()?;
    let wallet_handle = get_wallet_handle();

    let issuer_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;
    let issuer_verkey = settings::get_config_value(settings::CONFIG_INSTITUTION_VERKEY)?;

    let issuer_did =
        if !qualifier::is_fully_qualified(&issuer_did) {
            qualifier::qualify("did", &get_namespace(), &issuer_did)
        } else {
            issuer_did.to_string()
        };

    let (namespace, txn_bytes, signature_spec, bytes_to_sign, _) = match txn_type {
        TxnTypes::DID => {
            vdr::prepare_did(&vdr.vdr, txn_data, &issuer_did, None)
        }
        TxnTypes::Schema => {
            vdr::prepare_schema(&vdr.vdr, txn_data, &issuer_did, None)
        }
        TxnTypes::CredDef => {
            vdr::prepare_cred_def(&vdr.vdr, txn_data, &issuer_did, None)
        }
    }.wait().map_err(VcxError::from)?;

    let signature = crypto::sign(wallet_handle, &issuer_verkey, &bytes_to_sign)
        .wait()
        .map_err(VcxError::from)?;

    vdr::submit_txn(&vdr.vdr, &namespace, &txn_bytes, &signature_spec, &signature, None)
        .wait()
        .map_err(VcxError::from)
}

pub fn sign_and_submit_raw_txn(txn: &str) -> VcxResult<String> {
    let vdr: &VDRInfo = get_vdr()?;
    let wallet_handle = get_wallet_handle();
    let issuer_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    let signed_request =
        ledger::sign_request(wallet_handle, &issuer_did, &txn)
            .wait()
            .map_err(VcxError::from)?;

    vdr::submit_raw_txn(&vdr.vdr, DEFAULT_NETWORK, &signed_request.as_bytes())
        .wait()
        .map_err(VcxError::from)
}

pub fn get_txn_author_agreement() -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(crate::utils::constants::DEFAULT_AUTHOR_AGREEMENT.to_string()); }

    let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    let get_author_agreement_request = ledger::build_get_txn_author_agreement_request(Some(&did), None)
        .wait()?;

    let get_author_agreement_response = Request::submit(&get_author_agreement_request)?;

    let get_author_agreement_response = serde_json::from_str::<serde_json::Value>(&get_author_agreement_response)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("Could not parse Ledger response for GET TAA. Err: {:?}", err)))?;

    let mut author_agreement_data = get_author_agreement_response["result"]["data"].as_object()
        .map_or(json!({}), |data| json!(data));

    let get_acceptance_mechanism_request = ledger::build_get_acceptance_mechanisms_request(Some(&did), None, None)
        .wait()?;

    let get_acceptance_mechanism_response = Request::submit(&get_acceptance_mechanism_request)?;

    let get_acceptance_mechanism_response = serde_json::from_str::<serde_json::Value>(&get_acceptance_mechanism_response)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse, format!("Could not parse Ledger response for GET TAA AML. Err: {:?}", err)))?;

    if let Some(aml) = get_acceptance_mechanism_response["result"]["data"]["aml"].as_object() {
        author_agreement_data["aml"] = json!(aml);
    }

    Ok(author_agreement_data.to_string())
}

pub fn get_role(did: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() { return Ok(settings::DEFAULT_ROLE.to_string()); }

    let get_nym_req = Request::get_nym(None, &did)?;
    let get_nym_resp = Request::submit(&get_nym_req)?;

    let get_nym_resp: serde_json::Value = serde_json::from_str(&get_nym_resp)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse,
                                          format!("Could not parse Ledger response for GET_NYM. Err: {:?}", err)))?;

    let data: serde_json::Value = serde_json::from_str(&get_nym_resp["result"]["data"].as_str().unwrap_or("{}"))
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse,
                                          format!("Could not parse Ledger response for GET_NYM. Err: {:?}", err)))?;

    let role = data["role"].as_str().unwrap_or("null").to_string();
    Ok(role)
}

pub fn endorse_transaction(transaction_json: &str) -> VcxResult<()> {
    debug!("Ledger endorsing transaction");

    //TODO Potentially VCX should handle case when endorser would like to pay fee
    if settings::indy_mocks_enabled() { return Ok(()); }

    let submitter_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID)?;

    _verify_transaction_can_be_endorsed(transaction_json, &submitter_did)?;

    let transaction = Request::multisign(&submitter_did, transaction_json)?;
    let response = Request::submit(&transaction)?;

    match Response::parse(&response)? {
        TransactionResponse::Reply(_) => Ok(()),
        TransactionResponse::Reject(res) | TransactionResponse::ReqNACK(res) =>
            Err(VcxError::from_msg(VcxErrorKind::InvalidLedgerResponse,
                                   format!("Could not submit transaction on the Ledger. Response: {:?}", res)))?
    }
}

fn _verify_transaction_can_be_endorsed(transaction_json: &str, _did: &str) -> VcxResult<()> {
    let transaction: Transaction = serde_json::from_str(transaction_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("{:?}", err)))?;

    let transaction_endorser = transaction.endorser
        .ok_or(VcxError::from_msg(VcxErrorKind::InvalidJson, "Transaction cannot be endorsed: endorser DID is not set."))?;

    if transaction_endorser != _did {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson,
                                      format!("Transaction cannot be endorsed: transaction endorser DID `{}` and sender DID `{}` are different", transaction_endorser, _did)));
    }

    let identifier = transaction.identifier.as_str();
    if transaction.signature.is_none() && !transaction.signatures.as_ref().map(|signatures| signatures.contains_key(identifier)).unwrap_or(false) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidJson,
                                      format!("Transaction cannot be endorsed: the author must sign the transaction.")));
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::devsetup::*;
    #[cfg(feature = "pool_tests")]
    use crate::utils::constants::*;

    #[test]
    fn test_verify_transaction_can_be_endorsed() {
        let _setup = SetupDefaults::init();

        // success
        let transaction = r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "signature": "gkVDhwe2", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "NcYxiDXkpYi6ov5FcYDi1e").is_ok());

        // no author signature
        let transaction = r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "NcYxiDXkpYi6ov5FcYDi1e").is_err());

        // different endorser did
        let transaction = r#"{"reqId":1, "identifier": "EbP4aYNeTHL6q385GuVpRV", "endorser": "NcYxiDXkpYi6ov5FcYDi1e"}"#;
        assert!(_verify_transaction_can_be_endorsed(transaction, "EbP4aYNeTHL6q385GuVpRV").is_err());
    }

    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_endorse_transaction() {
        let _setup = SetupLibraryWalletPoolZeroFees::init();

        use crate::utils::libindy::payments::add_new_did;

        let (author_did, _) = add_new_did(None);
        let (endorser_did, _) = add_new_did(Some("ENDORSER"));

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &author_did);
        let schema_request = Request::schema(SCHEMA_DATA).unwrap();
        let schema_request = ledger::append_request_endorser(&schema_request, &endorser_did).wait().unwrap();
        let schema_request = Request::multisign(&author_did, &schema_request).unwrap();

        settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &endorser_did);
        endorse_transaction(&schema_request).unwrap();
    }
}