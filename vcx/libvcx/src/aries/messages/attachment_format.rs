use crate::aries::messages::attachment::AttachmentId;
use crate::error::{VcxResult, VcxError, VcxErrorKind};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AttachmentFormats(pub Vec<AttachmentFormat>);

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct AttachmentFormat {
    pub attach_id: AttachmentId,
    pub format: AttachmentFormatTypes,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum AttachmentFormatTypes {
    /// indy
    #[serde(rename = "hlindy/cred-abstract@v2.0")]
    IndyCredentialOffer,
    #[serde(rename = "hlindy/cred-req@v2.0")]
    IndyCredentialRequest,
    #[serde(rename = "hlindy/cred@v2.0")]
    IndyCredential,
    #[serde(rename = "hlindy/proof-req@v2.0")]
    IndyProofRequest,
    #[serde(rename = "hlindy/proof@v2.0")]
    IndyProof,
}

impl AttachmentFormats {
    pub fn new() -> AttachmentFormats {
        AttachmentFormats::default()
    }

    pub fn add(&mut self, attach_id: AttachmentId, format: AttachmentFormatTypes) {
        self.0.push(AttachmentFormat {
            attach_id,
            format,
        });
    }

    pub fn find(&self, attach_id: &AttachmentId) -> VcxResult<&AttachmentFormat> {
        let attach_id = attach_id.clone();
        self.0
            .iter()
            .find(|format| format.attach_id == attach_id)
            .ok_or(VcxError::from_msg(
                VcxErrorKind::InvalidAttachmentEncoding,
                format!("Unable to find attachment format for id: {:?}", attach_id))
            )
    }
}

impl Default for AttachmentFormatTypes {
    fn default() -> AttachmentFormatTypes {
        AttachmentFormatTypes::IndyCredential
    }
}
