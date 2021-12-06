use serde_json::{map::Map};

use crate::error::prelude::*;
use crate::settings;

pub fn ensure_credential_definition_contains_offered_attributes(cred_def_json: &str, attributes: Vec<&String>) -> VcxResult<()> {
    if settings::indy_mocks_enabled() { return Ok(()); }

    /*
        This check MUST have been done in URSA/Libindy but it is missing.
        Without this check we are able to issue a credential containing only part of fields but next
        we will get an error during proof generation.
    */

    let indy_cred_def: serde_json::Value = serde_json::from_str(cred_def_json)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::CredentialDefinitionNotFound,
                                          format!("Cannot parse Credential Definition from JSON string. Err: {:?}", err)))?;

    let cred_def_attributes: Map<String, serde_json::Value> =
        indy_cred_def["value"]["primary"]["r"].as_object().cloned()
            .ok_or(VcxError::from_msg(VcxErrorKind::CredentialDefinitionNotFound,
                                      "Cannot parse Credential Definition from JSON string. Err: The list of attributes not found"))?
            .into_iter()
            .filter(|(key, _)| !key.as_str().eq("master_secret")) // we have to omit `master_secret`
            .map(|(key, value)| (attr_common_view(&key), value))
            .collect();

    if attributes.len() != cred_def_attributes.len() || attributes.iter().any(|k| !cred_def_attributes.contains_key(attr_common_view(k).as_str())) {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidCredentialOffer,
                                      format!(
                                          "The list of attributes in Credential Offer \"{:?}\" does not match to the list of attributes in Credential Definition \"{:?}\"",
                                          cred_def_attributes.iter().map(|(key, _)| key.clone()).collect::<Vec<String>>(),
                                          attributes)));
    }

    Ok(())
}

pub fn attr_common_view(attr: &str) -> String {
    attr.replace(" ", "").to_lowercase()
}