use std::convert::TryInto;
use serde::{de, Deserialize, Deserializer};

use crate::aries::messages::issuance::v10::credential::Credential as CredentialV1;
use crate::aries::messages::issuance::v20::credential::Credential as CredentialV2;
use crate::aries::messages::ack::PleaseAck;
use crate::aries::messages::attachment::Attachments;
use crate::aries::messages::issuance::credential_offer::CredentialOffer;
use crate::error::prelude::*;
use crate::error::{VcxResult, VcxError};
use crate::aries::messages::thread::Thread;
use crate::agent::messages::payload::PayloadKinds;
use crate::legacy::messages::issuance::credential::CredentialMessage;
use crate::utils::libindy::types::Credential as IndyCredential;
use crate::utils::libindy::types::CredentialOffer as IndyCredentialOffer;
use crate::aries::messages::a2a::message_type::{MessageType, MessageTypeVersion};

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum Credential {
    V1(CredentialV1),
    V2(CredentialV2),
}

impl Credential {
    pub fn type_(&self) -> &MessageType {
        match self {
            Credential::V1(credential) => &credential.type_,
            Credential::V2(credential) => &credential.type_,
        }
    }

    pub fn please_ack(&self) -> Option<&PleaseAck> {
        match self {
            Credential::V1(credential) => credential.please_ack.as_ref(),
            Credential::V2(credential) => credential.please_ack.as_ref(),
        }
    }

    pub fn thread(&self) -> &Thread {
        match self {
            Credential::V1(credential) => &credential.thread,
            Credential::V2(credential) => &credential.thread,
        }
    }

    pub fn credentials_attach(&self) -> &Attachments {
        match self {
            Credential::V1(credential) => &credential.credentials_attach,
            Credential::V2(credential) => &credential.credentials_attach,
        }
    }

    pub fn ensure_match_offer(&self, credential_offer: &CredentialOffer) -> VcxResult<()> {
        let credential_preview = credential_offer.credentials_preview();
        let (_, attachment_content) = self.credentials_attach().content()?;

        let indy_cred: IndyCredential = serde_json::from_str(&attachment_content)
            .map_err(|err| VcxError::from_msg(
                VcxErrorKind::InvalidCredential,
                format!("Cannot parse Credential message from JSON string. Err: {:?}", err),
            ))?;

        let indy_offer: IndyCredentialOffer = serde_json::from_str(&attachment_content)
            .map_err(|err| VcxError::from_msg(
                VcxErrorKind::InvalidCredentialOffer,
                format!("Cannot parse Credential Offer message from JSON string. Err: {:?}", err),
            ))?;

        if indy_cred.schema_id != indy_offer.schema_id {
            return Err(VcxError::from_msg(
                VcxErrorKind::InvalidCredential,
                format!("Invalid Credential: Credential `schema_id` \"{}\" does not match to `schema_id` \"{}\" in Credential Offer.",
                        indy_cred.schema_id, indy_offer.schema_id),
            ));
        }

        if indy_cred.cred_def_id != indy_offer.cred_def_id {
            return Err(VcxError::from_msg(
                VcxErrorKind::InvalidCredential,
                format!("Invalid Credential: Credential `cred_def_id` \"{}\" does not match to `cred_def_id` \"{}\" in Credential Offer.",
                        indy_cred.schema_id, indy_offer.schema_id),
            ));
        }

        for attribute in credential_preview.attributes.iter() {
            let received_cred_attribute = indy_cred.values.0.get(&attribute.name)
                .ok_or(VcxError::from_msg(
                    VcxErrorKind::InvalidCredential,
                    format!("Invalid Credential: Cannot find \"{}\" attribute existing in the original Credential Offer.", attribute.name),
                ))?;

            if !received_cred_attribute.raw.eq(&attribute.value) {
                return Err(VcxError::from_msg(
                    VcxErrorKind::InvalidCredential,
                    format!("Invalid Credential: The value of \"{}\" attribute in Credential \
                                              does not match to the value \"{}\" of this attribute in the original Credential Offer.",
                            received_cred_attribute.raw, &attribute.value),
                ));
            }
        }

        Ok(())
    }


    pub fn from_thread(&self, id: &str) -> bool {
        match self {
            Credential::V1(credential) => credential.from_thread(id),
            Credential::V2(credential) => credential.from_thread(id),
        }
    }
}

impl TryInto<Credential> for CredentialMessage {
    type Error = VcxError;

    fn try_into(self) -> Result<Credential, Self::Error> {
        Ok(
            Credential::V1(
                CredentialV1::create()
                    .set_thread_id(&self.claim_offer_id)
                    .set_credential(self.libindy_cred)?
            )
        )
    }
}

impl TryInto<CredentialMessage> for Credential {
    type Error = VcxError;

    fn try_into(self) -> Result<CredentialMessage, Self::Error> {
        let (_, indy_credential_json) = self.credentials_attach().content()?;

        let indy_credential: ::serde_json::Value = ::serde_json::from_str(&indy_credential_json)
            .map_err(|err| VcxError::from_msg(
                VcxErrorKind::InvalidCredential,
                format!("Cannot deserialize Indy Credential: {:?}", err),
            ))?;

        Ok(CredentialMessage {
            msg_type: PayloadKinds::Cred.name().to_string(),
            libindy_cred: indy_credential_json,
            claim_offer_id: self.thread().thid.clone().unwrap_or_default(),
            cred_revoc_id: None,
            revoc_reg_delta_json: None,
            version: String::from("0.1"),
            from_did: String::new(),
            cred_def_id: indy_credential["cred_def_id"].as_str().map(String::from).unwrap_or_default(),
            rev_reg_def_json: String::new(),
        })
    }
}

deserialize_v1_v2_message!(Credential, CredentialV1, CredentialV2);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::issuance::v10::credential_offer::tests::_credential_offer;
    use crate::aries::messages::issuance::v10::credential::tests::_credential as _credential_v1;
    use crate::aries::messages::issuance::credential_preview::CredentialValue;

    pub fn _credential() -> Credential {
        Credential::V1(_credential_v1())
    }

    #[test]
    fn test_credential_match_offer() {
        // credential match offer
        _credential()
            .ensure_match_offer(&CredentialOffer::V1(_credential_offer()))
            .unwrap();

        // credential does not match offer - attribute not found in credential
        let mut offer = _credential_offer();
        offer.credential_preview.attributes.push(CredentialValue {
            name: "other_attribute".to_string(),
            value: json!("value"),
            _type: None,
        });
        _credential()
            .ensure_match_offer(&CredentialOffer::V1(offer))
            .unwrap_err();

        // credential does not match offer - different value in credential
        let mut offer = _credential_offer();
        offer.credential_preview.attributes[0].value = json!("other_value");
        _credential()
            .ensure_match_offer(&CredentialOffer::V1(offer))
            .unwrap_err();
    }
}