use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::issuance::credential_preview::CredentialPreviewData;
use crate::aries::messages::mime_type::MimeType;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::error::VcxResult;
use crate::aries::messages::thread::Thread;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CredentialProposal {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub credential_proposal: CredentialPreviewData,
    pub schema_id: String,
    pub cred_def_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>
}

impl CredentialProposal {
    pub fn create() -> Self {
        CredentialProposal::default()
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    pub fn set_schema_id(mut self, schema_id: String) -> Self {
        self.schema_id = schema_id;
        self
    }

    pub fn set_cred_def_id(mut self, cred_def_id: String) -> Self {
        self.cred_def_id = cred_def_id;
        self
    }

    pub fn add_credential_preview_data(mut self, name: &str, value: &serde_json::Value, mime_type: MimeType) -> VcxResult<CredentialProposal> {
        self.credential_proposal = self.credential_proposal.add_value(name, value, mime_type)?;
        Ok(self)
    }

    pub fn set_thread_id(mut self, id: &str) -> Self {
        self.thread = Some(Thread::new().set_thid(id.to_string()));
        self
    }
}

impl Default for CredentialProposal {
    fn default() -> CredentialProposal {
        CredentialProposal {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::CredentialIssuance,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::PROPOSE_CREDENTIAL.to_string()
            },
            comment: Default::default(),
            credential_proposal: Default::default(),
            schema_id: Default::default(),
            cred_def_id: Default::default(),
            thread: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::issuance::v10::credential_offer::tests::{thread, _value, thread_id};

    fn _attachment() -> ::serde_json::Value {
        json!({"credential offer": {}})
    }

    fn _comment() -> Option<String> {
        Some(String::from("comment"))
    }

    fn _schema_id() -> String { String::from("schema:id") }

    fn _cred_def_id() -> String { String::from("cred_def_id:id") }

    fn _credential_preview_data() -> CredentialPreviewData {
        let (name, value) = _value();

        CredentialPreviewData::new()
            .add_value(name,  &json!(value), MimeType::Plain).unwrap()
    }

    pub fn _credential_proposal() -> CredentialProposal {
        CredentialProposal {
            id: MessageId::id(),
            comment: _comment(),
            credential_proposal: _credential_preview_data(),
            schema_id: _schema_id(),
            thread: Some(thread()),
            cred_def_id: _cred_def_id(),
            ..CredentialProposal::default()
        }
    }

    #[test]
    fn test_credential_proposal_build_works() {
        let (name, value) = _value();

        let credential_proposal: CredentialProposal = CredentialProposal::create()
            .set_comment(_comment())
            .set_thread_id(&thread_id())
            .set_cred_def_id(_cred_def_id())
            .set_schema_id(_schema_id())
            .add_credential_preview_data(name,  &json!(value), MimeType::Plain).unwrap();

        assert_eq!(_credential_proposal(), credential_proposal);
        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/propose-credential","comment":"comment","cred_def_id":"cred_def_id:id","credential_proposal":{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/credential-preview","attributes":[{"name":"attribute","value":"value"}]},"schema_id":"schema:id","~thread":{"received_orders":{},"sender_order":0,"thid":"testid"}}"#;
        assert_eq!(expected, json!(credential_proposal).to_string());
    }
}