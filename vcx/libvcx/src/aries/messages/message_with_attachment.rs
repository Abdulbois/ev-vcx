use serde_json;

use crate::error::prelude::*;
use crate::aries::messages::attachment::Attachments;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct MessageWithAttachment {
    #[serde(rename = "filters~attach")]
    pub filters_attach: Option<Attachments>,
    #[serde(rename = "proposal~attach")]
    pub proposal_attach: Option<Attachments>,
    #[serde(rename = "did_doc~attach")]
    pub did_doc_attach: Option<Attachments>,
    #[serde(rename = "offers~attach")]
    pub offers_attach: Option<Attachments>,
    #[serde(rename = "request~attach")]
    pub request_attach: Option<Attachments>,
    #[serde(rename = "requests~attach")]
    pub requests_attach: Option<Attachments>,
    #[serde(rename = "credentials~attach")]
    pub credentials_attach: Option<Attachments>,
    #[serde(rename = "~attach")]
    pub attach: Option<Attachments>,
    #[serde(rename = "diagram~attach")]
    pub diagram_attach: Option<Attachments>,
    #[serde(rename = "agent~attach")]
    pub messages_attach: Option<Attachments>,
    #[serde(rename = "request_presentations~attach")]
    pub request_presentations_attach: Option<Attachments>,
    #[serde(rename = "presentations~attach")]
    pub presentations_attach: Option<Attachments>,
    #[serde(rename = "img~attach")]
    pub img_attach: Option<Attachments>,
}

pub fn extract_attached_message(message: &str) -> VcxResult<String> {
    trace!("Attachments::extract_attached_message >>>");
    debug!("Attachments: extracting attachment from message");

    let message_with_attachment: MessageWithAttachment = serde_json::from_str(message)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Unable to parse MessageWithAttachment from JSON string. Err: {:?}", err),
        ))?;

    let attachment: Attachments = message_with_attachment.filters_attach
        .or(message_with_attachment.proposal_attach)
        .or(message_with_attachment.did_doc_attach)
        .or(message_with_attachment.offers_attach)
        .or(message_with_attachment.request_attach)
        .or(message_with_attachment.requests_attach)
        .or(message_with_attachment.credentials_attach)
        .or(message_with_attachment.attach)
        .or(message_with_attachment.diagram_attach)
        .or(message_with_attachment.messages_attach)
        .or(message_with_attachment.request_presentations_attach)
        .or(message_with_attachment.presentations_attach)
        .or(message_with_attachment.img_attach)
        .ok_or(VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            "Message does not contain attachment",
        ))?;

    let (_, content) = attachment.content()?;
    trace!("Attachments::extract_attached_message <<< content: {:?}", secret!(content));
    Ok(content)
}


#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_extract_attached_message() {
        let message = r#"{"request_presentations~attach": [{"@id": "libindy-request-presentation-0", "data": {"base64": "eyJuYW1lIjoicHJvb2ZfZnJvbV9hbGljZSIsIm5vbl9yZXZva2VkIjpudWxsLCJub25jZSI6Ijc3MzQ4MDc1MzM0NDk3MDI5ODY2MDgiLCJyZXF1ZXN0ZWRfYXR0cmlidXRlcyI6eyJhdHRyaWJ1dGVfMCI6eyJuYW1lIjoiTWVtYmVySUQifX0sInJlcXVlc3RlZF9wcmVkaWNhdGVzIjp7fSwidmVyIjoiMS4wIiwidmVyc2lvbiI6IjEuMCJ9"}, "mime-type": "application/json"}]}"#;
        let attached_message = extract_attached_message(message).unwrap();
        let expected_message = r#"{"name":"proof_from_alice","non_revoked":null,"nonce":"7734807533449702986608","requested_attributes":{"attribute_0":{"name":"MemberID"}},"requested_predicates":{},"ver":"1.0","version":"1.0"}"#;
        assert_eq!(attached_message, expected_message);
    }
}