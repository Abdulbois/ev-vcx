use crate::aries::messages::a2a::{A2AMessage, MessageId};
use crate::aries::messages::thread::Thread;
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::aries::messages::proof_presentation::presentation_preview::PresentationPreview;
use crate::utils::libindy::anoncreds::proof_request::{AttributeInfo, PredicateInfo, Restrictions};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PresentationProposal {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub presentation_proposal: PresentationPreview,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
}

impl PresentationProposal {
    pub fn create() -> Self {
        PresentationProposal::default()
    }

    pub fn set_comment(mut self, comment: String) -> Self {
        self.comment = Some(comment);
        self
    }

    pub fn set_presentation_preview(mut self, presentation_preview: PresentationPreview) -> PresentationProposal {
        self.presentation_proposal = presentation_preview;
        self
    }

    pub fn set_thread_id(mut self, id: &str) -> Self {
        self.thread = Some(Thread::new().set_thid(id.to_string()));
        self
    }

    pub fn set_thread(mut self, thread: Thread) -> Self {
        self.thread = Some(thread);
        self
    }

    pub fn to_proof_request_requested_attributes(&self) -> Vec<AttributeInfo> {
        self.presentation_proposal.attributes
            .iter()
            .map(|attribute| AttributeInfo {
                name: Some(attribute.name.clone()),
                names: None,
                restrictions: attribute.cred_def_id
                    .as_ref()
                    .map(|cred_def_id|
                        Restrictions::V2(json!({
                        "cred_def_id": cred_def_id
                    }))
                    ),
                non_revoked: None,
                self_attest_allowed: None,
            })
            .collect()
    }

    pub fn to_proof_request_requested_predicates(&self) -> Vec<PredicateInfo> {
        self.presentation_proposal.predicates
            .iter()
            .map(|predicate| PredicateInfo {
                name: predicate.name.clone(),
                p_type: predicate.predicate.clone(),
                p_value: predicate.threshold as i32,
                restrictions: predicate.cred_def_id
                    .as_ref()
                    .map(|cred_def_id|
                        Restrictions::V2(json!({
                        "cred_def_id": cred_def_id
                    }))
                    ),
                non_revoked: None,
            })
            .collect()
    }
}

impl Default for PresentationProposal {
    fn default() -> PresentationProposal {
        PresentationProposal {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::PresentProof,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::PROPOSE_PRESENTATION.to_string(),
            },
            comment: Default::default(),
            presentation_proposal: Default::default(),
            thread: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::proof_presentation::v10::presentation_request::tests::{thread, thread_id};
    use crate::aries::messages::proof_presentation::presentation_preview::tests::_presentation_preview;

    fn _attachment() -> ::serde_json::Value {
        json!({"presentation": {}})
    }

    fn _comment() -> String {
        String::from("comment")
    }

    pub fn _presentation_proposal() -> PresentationProposal {
        PresentationProposal {
            id: MessageId::id(),
            comment: Some(_comment()),
            thread: Some(thread()),
            presentation_proposal: _presentation_preview(),
            ..PresentationProposal::default()
        }
    }

    #[test]
    fn test_presentation_proposal_build_works() {
        let presentation_proposal: PresentationProposal = PresentationProposal::default()
            .set_comment(_comment())
            .set_thread_id(&thread_id())
            .set_presentation_preview(_presentation_preview());

        assert_eq!(_presentation_proposal(), presentation_proposal);

        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/propose-presentation","comment":"comment","presentation_proposal":{"@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/present-proof/1.0/presentation-preview","attributes":[{"cred_def_id":"BzCbsNYhMrjHiqZDTUASHg:3:CL:1234:tag","name":"account"}],"predicates":[]},"~thread":{"received_orders":{},"sender_order":0,"thid":"testid"}}"#;
        assert_eq!(expected.to_string(), json!(presentation_proposal).to_string())
    }
}
