use std::convert::TryInto;
use std::collections::HashMap;
use serde_json::Value;
use serde::{de, Deserialize, Deserializer};

use crate::aries::messages::issuance::v10::credential_offer::CredentialOffer as CredentialOfferV1;
use crate::aries::messages::issuance::v20::credential_offer::CredentialOffer as CredentialOfferV2;
use crate::aries::messages::connection::service::Service;
use crate::aries::messages::mime_type::MimeType;
use crate::aries::messages::attachment::Attachments;
use crate::aries::messages::issuance::credential_preview::{CredentialPreviewData, CredentialValue};
use crate::error::{VcxResult, VcxError, VcxErrorKind};
use crate::legacy::messages::issuance::credential_offer::CredentialOffer as ProprietaryCredentialOffer;
use crate::aries::messages::thread::Thread;
use crate::agent::messages::payload::PayloadKinds;
use crate::utils::libindy::anoncreds::utils::ensure_credential_definition_contains_offered_attributes;
use crate::utils::libindy::anoncreds::types::CredentialOffer as IndyCredentialOffer;
use crate::aries::messages::a2a::message_type::{MessageType, MessageTypeVersion};
use crate::aries::messages::alias::Alias;

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum CredentialOffer {
    V1(CredentialOfferV1),
    V2(CredentialOfferV2),
}

impl CredentialOffer {
    pub fn add_credential_preview_data(self, name: &str, value: &Value, mime_type: MimeType) -> VcxResult<CredentialOffer> {
        match self {
            CredentialOffer::V1(credential_offer) => {
                Ok(CredentialOffer::V1(credential_offer.add_credential_preview_data(name, value, mime_type)?))
            }
            CredentialOffer::V2(credential_offer) => {
                Ok(CredentialOffer::V2(credential_offer.add_credential_preview_data(name, value, mime_type)?))
            }
        }
    }

    pub fn set_thread_id(self, thid: &str) -> Self {
        match self {
            CredentialOffer::V1(offer) => CredentialOffer::V1(offer.set_thread_id(thid)),
            CredentialOffer::V2(offer) => CredentialOffer::V2(offer.set_thread_id(thid))
        }
    }

    pub fn id(&self) -> String {
        match self {
            CredentialOffer::V1(offer) => offer.id.to_string(),
            CredentialOffer::V2(offer) => offer.id.to_string(),
        }
    }

    pub fn type_(&self) -> &MessageType {
        match self {
            CredentialOffer::V1(offer) => &offer.type_,
            CredentialOffer::V2(offer) => &offer.type_,
        }
    }

    pub fn comment(&self) -> Option<String> {
        match self {
            CredentialOffer::V1(offer) => offer.comment.clone(),
            CredentialOffer::V2(offer) => offer.comment.clone(),
        }
    }

    pub fn alias(&self) -> Option<&Alias> {
        match self {
            CredentialOffer::V1(offer) => offer.alias.as_ref(),
            CredentialOffer::V2(offer) => offer.alias.as_ref(),
        }
    }

    pub fn thread(&self) -> Option<&Thread> {
        match self {
            CredentialOffer::V1(offer) => offer.thread.as_ref(),
            CredentialOffer::V2(offer) => offer.thread.as_ref(),
        }
    }

    pub fn service(&self) -> Option<&Service> {
        match self {
            CredentialOffer::V1(credential_offer) => credential_offer.service.as_ref(),
            CredentialOffer::V2(credential_offer) => credential_offer.service.as_ref()
        }
    }

    pub fn offer_attach(&self) -> &Attachments {
        match self {
            CredentialOffer::V1(credential_offer) => &credential_offer.offers_attach,
            CredentialOffer::V2(credential_offer) => &credential_offer.offers_attach
        }
    }

    pub fn credentials_preview(&self) -> &CredentialPreviewData {
        match self {
            CredentialOffer::V1(credential_offer) => &credential_offer.credential_preview,
            CredentialOffer::V2(credential_offer) => &credential_offer.credential_preview,
        }
    }

    pub fn ensure_match_credential_definition(&self, cred_def_json: &str) -> VcxResult<()> {
        let cred_offer_attributes =
            self.credentials_preview()
                .attributes
                .iter()
                .map(|value| &value.name)
                .collect();

        ensure_credential_definition_contains_offered_attributes(cred_def_json, cred_offer_attributes)
    }

    pub fn append_credential_preview(mut self, data: &str) -> VcxResult<CredentialOffer> {
        let cred_values: HashMap<String, Value> = serde_json::from_str(data)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidAttributesStructure,
                                              format!("Cannot parse Credential Preview from JSON string. Err: {:?}", err)))?;

        for (key, value) in cred_values {
            self = self.add_credential_preview_data(&key, &value, MimeType::Plain)?;
        }

        trace!("Issuer::InitialState::append_credential_preview <<<");
        Ok(self)
    }

    pub fn parse(offer: &str) -> VcxResult<String> {
        let offer: CredentialOffer = ::serde_json::from_str(offer)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidCredentialOffer,
                                              format!("Cannot parse CredentialOffer from `offer` JSON string. Err: {:?}", err)))?;

        let thid = offer.thread().as_ref()
            .and_then(|thread| thread.thid.clone())
            .unwrap_or_else(|| offer.id());

        let (_, cred_offer) = offer.offer_attach().content()?;

        let indy_offer: IndyCredentialOffer = serde_json::from_str(&cred_offer)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidCredentialOffer,
                                              format!("Cannot parse Indy Credential Offer from JSON string. Err: {:?}", err)))?;

        let name = indy_offer.extract_schema_name()
            .or(offer.alias().map(|alias| alias.label.clone()))
            .or(offer.comment())
            .unwrap_or("Credential".to_string());

        let info = CredentialOfferInfo {
            name,
            attributes: offer.credentials_preview().clone().attributes,
            cred_def_id: indy_offer.cred_def_id,
            schema_id: indy_offer.schema_id,
            thid,
        };

        Ok(json!(info).to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CredentialOfferInfo {
    pub name: String,
    pub attributes: Vec<CredentialValue>,
    pub cred_def_id: String,
    pub schema_id: String,
    pub thid: String,
}

impl TryInto<CredentialOffer> for ProprietaryCredentialOffer {
    type Error = VcxError;

    fn try_into(self) -> Result<CredentialOffer, Self::Error> {
        let mut credential_preview = CredentialPreviewData::new();

        for (key, value) in self.credential_attrs {
            credential_preview = credential_preview.add_value(&key, &value, MimeType::Plain)?;
        }

        Ok(
            CredentialOffer::V1(
                CredentialOfferV1::create()
                    .set_id(self.thread_id.unwrap_or_default())
                    .set_credential_preview_data(credential_preview)?
                    .set_offers_attach(&self.libindy_offer)?
            )
        )
    }
}

impl TryInto<ProprietaryCredentialOffer> for CredentialOffer {
    type Error = VcxError;

    fn try_into(self) -> Result<ProprietaryCredentialOffer, Self::Error> {
        let (_, indy_cred_offer_json) = self.offer_attach().content()?;

        let indy_cred_offer: ::serde_json::Value = ::serde_json::from_str(&indy_cred_offer_json)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidCredentialOffer, format!("Cannot deserialize Indy Offer: {:?}", err)))?;

        let mut credential_attrs: ::serde_json::Map<String, ::serde_json::Value> = ::serde_json::Map::new();

        for attr in self.credentials_preview().attributes.iter() {
            credential_attrs.insert(attr.name.clone(), attr.value.clone());
        }

        let thid = self.thread().and_then(|thread| thread.thid.clone()).unwrap_or(self.id());

        Ok(ProprietaryCredentialOffer {
            msg_type: PayloadKinds::CredOffer.name().to_string(),
            version: String::from("0.1"),
            to_did: String::new(),
            from_did: String::new(),
            credential_attrs,
            schema_seq_no: 0,
            claim_name: self.comment().unwrap_or("".to_string()),
            claim_id: String::new(),
            msg_ref_id: None,
            cred_def_id: indy_cred_offer["cred_def_id"].as_str().map(String::from).unwrap_or_default(),
            libindy_offer: indy_cred_offer_json,
            thread_id: Some(thid),
        })
    }
}

deserialize_v1_v2_message!(CredentialOffer, CredentialOfferV1, CredentialOfferV2);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::utils::constants::CRED_DEF_JSON;
    pub use crate::aries::messages::issuance::v10::credential_offer::tests::{thread, thread_id, _value};
    use crate::aries::messages::issuance::v10::credential_offer::tests::_credential_offer as _credential_offer_v1;

    pub fn _credential_offer() -> CredentialOffer {
        CredentialOffer::V1(_credential_offer_v1())
    }

    #[test]
    fn test_credential_offer_match_cred_def_works() {
        // Credential Definition contains attributes: name, height, sex, age

        // Credential Offer contains less attributes than Credential Definition
        let credential_offer: CredentialOffer = CredentialOffer::V1(
            CredentialOfferV1::create()
                .set_credential_preview_data(
                    CredentialPreviewData::new()
                        .add_value("name", &json!("Test"), MimeType::Plain).unwrap()
                ).unwrap()
        );

        credential_offer.ensure_match_credential_definition(CRED_DEF_JSON).unwrap_err();

        // Credential Offer contains same attributes as Credential Definition
        let credential_offer: CredentialOffer = CredentialOffer::V1(
            CredentialOfferV1::create()
                .set_credential_preview_data(
                    CredentialPreviewData::new()
                        .add_value("name", &json!("Test"), MimeType::Plain).unwrap()
                        .add_value("height", &json!("Test"), MimeType::Plain).unwrap()
                        .add_value("sex", &json!("Test"), MimeType::Plain).unwrap()
                        .add_value("age", &json!("Test"), MimeType::Plain).unwrap()
                ).unwrap()
        );

        credential_offer.ensure_match_credential_definition(CRED_DEF_JSON).unwrap();

        // Credential Offer contains same attributes as Credential Definition but in different case
        let credential_offer: CredentialOffer = CredentialOffer::V1(
            CredentialOfferV1::create()
                .set_credential_preview_data(
                    CredentialPreviewData::new()
                        .add_value("NAME", &json!("Test"), MimeType::Plain).unwrap()
                        .add_value("Height", &json!("Test"), MimeType::Plain).unwrap()
                        .add_value("SEX", &json!("Test"), MimeType::Plain).unwrap()
                        .add_value("age", &json!("Test"), MimeType::Plain).unwrap()
                ).unwrap()
        );

        credential_offer.ensure_match_credential_definition(CRED_DEF_JSON).unwrap();

        // Credential Offer contains more attributes than Credential Definition
        let credential_offer: CredentialOffer = CredentialOffer::V1(
            CredentialOfferV1::create()
                .set_credential_preview_data(
                    CredentialPreviewData::new()
                        .add_value("name", &json!("Test"), MimeType::Plain).unwrap()
                        .add_value("height", &json!("Test"), MimeType::Plain).unwrap()
                        .add_value("sex", &json!("Test"), MimeType::Plain).unwrap()
                        .add_value("age", &json!("Test"), MimeType::Plain).unwrap()
                        .add_value("additional", &json!("Test"), MimeType::Plain).unwrap()
                ).unwrap()
        );

        credential_offer.ensure_match_credential_definition(CRED_DEF_JSON).unwrap_err();
    }

    #[test]
    fn test_parse_credential_offer(){
        let offer = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential","comment":"comment","credential_preview":{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/credential-preview","attributes":[{"name":"attribute","value":"value"}]},"offers~attach":[{"@id":"libindy-cred-offer-0","data":{"base64":"eyJjcmVkX2RlZl9pZCI6Ik5jWXhpRFhrcFlpNm92NUZjWURpMWU6MzpDTDpOY1l4aURYa3BZaTZvdjVGY1lEaTFlOjI6Z3Z0OjEuMDpUQUcxIiwic2NoZW1hX2lkIjoiTmNZeGlEWGtwWWk2b3Y1RmNZRGkxZToyOmd2dDoxLjAifQ=="},"mime-type":"application/json"}]}"#;
        let info = CredentialOffer::parse(offer).unwrap();
        let expected = CredentialOfferInfo {
            name: "gvt".to_string(),
            attributes: vec![CredentialValue {
                name: "attribute".to_string(),
                value: serde_json::Value::String("value".to_string()),
                _type: None
            }],
            cred_def_id: "NcYxiDXkpYi6ov5FcYDi1e:3:CL:NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0:TAG1".to_string(),
            schema_id: "NcYxiDXkpYi6ov5FcYDi1e:2:gvt:1.0".to_string(),
            thid: "testid".to_string()
        };
        assert_eq!(json!(expected).to_string(), info);
    }
}