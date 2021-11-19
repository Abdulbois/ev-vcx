pub mod holder_fsm;
pub mod messages;
pub mod states;

use self::{
    holder_fsm::HolderSM,
    messages::HolderMessages,
};
use crate::error::prelude::*;
use crate::aries::messages::{
    a2a::A2AMessage,
    issuance::{
        credential::Credential,
        credential_offer::CredentialOffer,
    },
    proof_presentation::{
        presentation_preview::PresentationPreview,
        presentation_proposal::PresentationProposal,
        v10::presentation_proposal::PresentationProposal as PresentationProposalV1,
    },
    error::ProblemReport,
};
use crate::connection::Connections;
use crate::utils::object_cache::Handle;
use crate::utils::libindy::{
    anoncreds::prover_get_credential,
    types::CredentialInfo,
};

// Holder

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Holder {
    holder_sm: HolderSM
}

impl Holder {
    pub fn create(credential_offer: CredentialOffer, source_id: &str) -> VcxResult<Holder> {
        trace!("Holder::holder_create_credential >>> credential_offer: {:?}, source_id: {:?}", credential_offer, source_id);
        debug!("Holder {}: Creating credential Holder state object", source_id);

        let holder_sm = HolderSM::new(credential_offer, source_id.to_string());

        Ok(Holder { holder_sm })
    }

    pub fn send_request(&mut self, connection_handle: Handle<Connections>) -> VcxResult<()> {
        trace!("Holder::send_request >>>");
        debug!("Holder {}: Sending credential request", self.get_source_id());
        self.step(HolderMessages::CredentialRequestSend(connection_handle))
    }

    pub fn send_reject(&mut self, connection_handle: Handle<Connections>, comment: Option<String>) -> VcxResult<()> {
        trace!("Holder::send_reject >>> comment: {:?}", comment);
        debug!("Holder {}: Sending credential reject", self.get_source_id());
        self.step(HolderMessages::CredentialRejectSend((connection_handle, comment)))
    }

    pub fn update_state(&mut self, msg: Option<String>) -> VcxResult<u32> {
        trace!("Holder: update_state >>> msg: {:?}", secret!(msg));
        debug!("Holder {}: Updating state", self.get_source_id());

        match msg {
            Some(msg) => {
                let message: A2AMessage = ::serde_json::from_str(&msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                                      format!("Cannot updated Holder state with agent: Message deserialization failed with: {:?}", err)))?;

                self.step(message.into())?;
            }
            None => {
                self.holder_sm = self.holder_sm.clone().update_state()?;
            }
        };

        let state = self.get_state();

        trace!("Holder::update_state <<< state: {:?}", state);
        Ok(state)
    }

    pub fn get_state(&self) -> u32 {
        self.holder_sm.state()
    }

    pub fn get_source_id(&self) -> String {
        self.holder_sm.get_source_id()
    }

    pub fn get_credential_offer(&self) -> VcxResult<CredentialOffer> {
        trace!("Holder::get_credential_offer >>>");
        debug!("Holder {}: Getting credential offer", self.get_source_id());
        self.holder_sm.get_credential_offer()
    }

    pub fn get_credential(&self) -> VcxResult<(String, Credential)> {
        trace!("Holder::get_credential >>>");
        debug!("Holder {}: Getting credential", self.get_source_id());
        self.holder_sm.get_credential()
    }

    pub fn delete_credential(&self) -> VcxResult<()> {
        debug!("Holder {}: Deleting credential", self.get_source_id());
        self.holder_sm.delete_credential()
    }

    pub fn get_presentation_proposal(&self) -> VcxResult<PresentationProposal> {
        trace!("Holder::get_presentation_proposal >>>");
        debug!("Credential {}: Building presentation proposal", self.get_source_id());

        let (cred_id, _) = self.get_credential()?;
        let credential_offer = self.get_credential_offer()?;

        let credential = prover_get_credential(&cred_id)?;

        let presentation_proposal = PresentationProposal::V1(
            PresentationProposalV1::create()
                .set_comment(credential_offer.comment().unwrap_or(String::from("Credential")))
                .set_presentation_preview(PresentationPreview::for_credential(&credential))
        );

        trace!("Credential::get_presentation_proposal <<< presentation_proposal: {:?}", presentation_proposal);
        Ok(presentation_proposal)
    }

    pub fn get_problem_report_message(&self) -> VcxResult<String> {
        trace!("Holder::get_problem_report_message >>>");
        debug!("Holder {}: Getting problem report message", self.get_source_id());

        let problem_report: Option<&ProblemReport> = self.holder_sm.problem_report();
        Ok(json!(&problem_report).to_string())
    }

    pub fn get_info(&self) -> VcxResult<String> {
        trace!("Holder::get_info >>>");
        debug!("Holder {}: Getting credential info", self.get_source_id());

        let info: CredentialInfo = self.holder_sm.get_info()?;
        Ok(json!(&info).to_string())
    }

    pub fn step(&mut self, message: HolderMessages) -> VcxResult<()> {
        self.holder_sm = self.holder_sm.clone().handle_message(message)?;
        Ok(())
    }

    pub fn get_credential_offer_message(connection_handle: Handle<Connections>, msg_id: &str) -> VcxResult<CredentialOffer> {
        trace!("Holder::get_credential_offer_message >>> connection_handle: {}, msg_id: {}", connection_handle, msg_id);
        debug!("Holder: Getting credential offer {} from the agent", msg_id);

        let message = connection_handle.get_message_by_id(msg_id.to_string())?;

        let credential_offer: CredentialOffer = match message {
            A2AMessage::CredentialOffer(credential_offer) => credential_offer,
            msg => {
                return Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse,
                                              format!("Message of different type has been received. Expected: CredentialOffer. Received: {:?}", msg)));
            }
        };

        trace!("Holder: get_credential_offer_message <<< credential_offer: {:?}", secret!(credential_offer));
        Ok(credential_offer)
    }

    pub fn get_credential_offer_messages(conn_handle: Handle<Connections>) -> VcxResult<Vec<CredentialOffer>> {
        trace!("Holder::get_credential_offer_messages >>>");
        debug!("Holder: Getting all credential offers from the agent");

        let msgs = conn_handle
            .get_messages()?
            .into_iter()
            .filter_map(|(_, a2a_message)| {
                match a2a_message {
                    A2AMessage::CredentialOffer(credential_offer) => {
                        Some(credential_offer)
                    }
                    _ => None
                }
            })
            .collect();

        trace!("Holder: get_credential_offer_messages <<<");
        Ok(msgs)
    }
}
