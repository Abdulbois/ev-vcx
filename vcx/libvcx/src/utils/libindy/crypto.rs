use futures::Future;

use crate::indy::{crypto, did};
use crate::settings;
use crate::error::prelude::*;
use crate::utils::libindy::LibindyMock;
use crate::utils::constants::*;
use crate::utils::libindy::wallet::get_wallet_handle;

pub fn prep_msg(sender_vk: &str, recipient_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() {
        let rc = LibindyMock::get_result();
        if rc != 0 { return Err(VcxError::from(VcxErrorKind::Common(rc))); };
        return Ok(Vec::from(msg).to_owned());
    }

    crypto::auth_crypt(crate::utils::libindy::wallet::get_wallet_handle(), sender_vk, recipient_vk, msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn prep_anonymous_msg(recipient_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::anon_crypt(recipient_vk, msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn parse_msg(recipient_vk: &str, msg: &[u8]) -> VcxResult<(String, Vec<u8>)> {
    if settings::indy_mocks_enabled() { return Ok((crate::utils::constants::VERKEY.to_string(), Vec::from(msg).to_owned())); }

    crypto::auth_decrypt(crate::utils::libindy::wallet::get_wallet_handle(), recipient_vk, msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn parse_anonymous_msg(recipient_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::anon_decrypt(crate::utils::libindy::wallet::get_wallet_handle(), recipient_vk, msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn sign(my_vk: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::sign(crate::utils::libindy::wallet::get_wallet_handle(), my_vk, msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn verify(vk: &str, msg: &[u8], signature: &[u8]) -> VcxResult<bool> {
    if settings::indy_mocks_enabled() { return Ok(true); }

    crypto::verify(vk, msg, signature)
        .wait()
        .map_err(VcxError::from)
}

pub fn pack_message(sender_vk: Option<&str>, receiver_keys: &str, msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(msg.to_vec()); }

    crypto::pack_message(crate::utils::libindy::wallet::get_wallet_handle(), msg, receiver_keys, sender_vk)
        .wait()
        .map_err(VcxError::from)
}

pub fn unpack_message(msg: &[u8]) -> VcxResult<Vec<u8>> {
    if settings::indy_mocks_enabled() { return Ok(Vec::from(msg).to_owned()); }

    crypto::unpack_message(crate::utils::libindy::wallet::get_wallet_handle(), msg)
        .wait()
        .map_err(VcxError::from)
}

pub fn create_key(seed: Option<&str>) -> VcxResult<String> {
    let key_json = json!({"seed": seed}).to_string();
    crypto::create_key(crate::utils::libindy::wallet::get_wallet_handle(), Some(&key_json))
        .wait()
        .map_err(VcxError::from)
}

pub fn create_and_store_my_did(seed: Option<&str>, method_name: Option<&str>) -> VcxResult<(String, String)> {
    if settings::indy_mocks_enabled() {
        return Ok((DID.to_string(), VERKEY.to_string()));
    }

    let my_did_json = json!({"seed": seed, "method_name": method_name});

    did::create_and_store_my_did(get_wallet_handle(), &my_did_json.to_string())
        .wait()
        .map_err(VcxError::from)
}

pub fn get_local_verkey(did: &str) -> VcxResult<String> {
    if settings::indy_mocks_enabled() {
        return Ok(VERKEY.to_string());
    }

    did::key_for_local_did(get_wallet_handle(), did)
        .wait()
        .map_err(VcxError::from)
}