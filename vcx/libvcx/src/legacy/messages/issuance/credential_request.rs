use crate::error::prelude::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct CredentialRequest {
    pub libindy_cred_req: String,
    pub libindy_cred_req_meta: String,
    pub cred_def_id: String,
    pub tid: String,
    pub to_did: String,
    pub from_did: String,
    pub version: String,
    pub mid: String,
    pub msg_ref_id: Option<String>,
}

impl CredentialRequest {
    pub fn new(did: &str) -> CredentialRequest {
        CredentialRequest {
            to_did: String::new(),
            from_did: did.to_string(),
            mid: String::new(),
            tid: String::new(),
            version: String::new(),
            libindy_cred_req: String::new(),
            libindy_cred_req_meta: String::new(),
            cred_def_id: String::new(),
            msg_ref_id: None
        }
    }
}

pub fn set_cred_req_ref_message(cred_request: &str, msg_id: &str) -> VcxResult<CredentialRequest> {
    trace!("set_cred_req_ref_message >>> cred_request: {:?}, msg_id: {:?}", secret!(cred_request), msg_id);

    let mut request: CredentialRequest = serde_json::from_str(&cred_request)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidCredentialRequest, format!("Cannot deserialize Credential Request: {}", err)))?;

    request.msg_ref_id = Some(msg_id.to_owned());

    Ok(request)
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use crate::utils::constants::{CREDENTIAL_REQ_STRING, CRED_REQ, CRED_REQ_META};
    use crate::utils::devsetup::*;

    fn create_credential_req() -> CredentialRequest {
        let _setup = SetupDefaults::init();

        crate::settings::set_defaults();
        let issuer_did = crate::settings::get_config_value(crate::settings::CONFIG_INSTITUTION_DID).unwrap();
        CredentialRequest::new(&issuer_did)
    }

    #[test]
    fn test_credential_request_struct() {
        let _setup = SetupDefaults::init();

        let req = create_credential_req();
        let issuer_did = crate::settings::get_config_value(crate::settings::CONFIG_INSTITUTION_DID).unwrap();
        assert_eq!(req.from_did, issuer_did);
    }

    #[test]
    fn test_serialize() {
        let _setup = SetupDefaults::init();

        let cred1: CredentialRequest = serde_json::from_str(CREDENTIAL_REQ_STRING).unwrap();
        let serialized = serde_json::to_string(&cred1).unwrap();
        assert_eq!(serialized, CREDENTIAL_REQ_STRING)
    }

    #[test]
    fn test_deserialize() {
        let _setup = SetupDefaults::init();

        let req: CredentialRequest = serde_json::from_str(CREDENTIAL_REQ_STRING).unwrap();
        assert_eq!(&req.libindy_cred_req, CRED_REQ);
    }

    #[test]
    fn test_create_credential_request_from_raw_message() {
        let _setup = SetupDefaults::init();

        let credential_req: CredentialRequest = serde_json::from_str(CREDENTIAL_REQ_STRING).unwrap();

        assert_eq!(credential_req.tid, "cCanHnpFAD");
        assert_eq!(credential_req.to_did, "BnRXf8yDMUwGyZVDkSENeq");
        assert_eq!(credential_req.from_did, "GxtnGN6ypZYgEqcftSQFnC");
        assert_eq!(credential_req.version, "0.1");
        assert_eq!(credential_req.mid, "");
        assert_eq!(&credential_req.libindy_cred_req, CRED_REQ);
        assert_eq!(&credential_req.libindy_cred_req_meta, CRED_REQ_META);
    }
}

