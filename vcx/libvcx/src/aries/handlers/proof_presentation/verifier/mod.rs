pub mod messages;
pub mod states;
pub mod verifier_fsm;

use self::{
    verifier_fsm::VerifierSM,
    messages::VerifierMessages,
};
use crate::error::prelude::*;
use crate::connection::Connections;
use crate::utils::object_cache::Handle;
use crate::aries::messages::{
    a2a::A2AMessage,
    proof_presentation::{
        presentation_request::PresentationRequest,
        presentation::Presentation,
        presentation_proposal::PresentationProposal,
    },
    error::ProblemReport,
};
use crate::utils::libindy::anoncreds::proof_request::ProofRequest;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Verifier {
    verifier_sm: VerifierSM
}

impl Verifier {
    pub fn create(source_id: String,
                  requested_attrs: String,
                  requested_predicates: String,
                  revocation_details: String,
                  name: String) -> VcxResult<Verifier> {
        trace!("Verifier::create >>> source_id: {:?}, requested_attrs: {:?}, requested_predicates: {:?}, revocation_details: {:?}, name: {:?}",
               source_id, secret!(requested_attrs), secret!(requested_predicates), secret!(revocation_details), secret!(name));
        debug!("Verifier {}: Creating Verifier state object", source_id);

        let presentation_request =
            ProofRequest::create()
                .set_name(name)
                .set_requested_attributes(requested_attrs)?
                .set_requested_predicates(requested_predicates)?
                .set_not_revoked_interval(revocation_details)?
                .set_nonce()?;

        Ok(Verifier {
            verifier_sm: VerifierSM::new(presentation_request, source_id),
        })
    }

    pub fn create_from_proposal(source_id: String, presentation_proposal: PresentationProposal) -> VcxResult<Verifier> {
        Ok(Verifier {
            verifier_sm: VerifierSM::new_from_proposal(presentation_proposal, source_id)
        })
    }

    pub fn get_source_id(&self) -> &String { self.verifier_sm.source_id() }

    pub fn state(&self) -> u32 {
        trace!("Verifier::state >>>");
        self.verifier_sm.state()
    }

    pub fn presentation_status(&self) -> u32 {
        trace!("Verifier::presentation_state >>>");
        debug!("Verifier {}: Getting presentation status", self.get_source_id());
        self.verifier_sm.presentation_status()
    }

    pub fn update_state(&mut self, message: Option<&str>) -> VcxResult<u32> {
        trace!("Verifier::update_state >>> message: {:?}", secret!(message));
        debug!("Verifier {}: Updating state", self.get_source_id());

        if !self.verifier_sm.has_transitions() { return Ok(self.state()); }

        if let Some(message_) = message {
            return self.update_state_with_message(message_);
        }

        let agent_info = match self.verifier_sm.get_agent_info() {
            Some(agent_info) => agent_info.clone(),
            None => {
                warn!("Could not update Verifier state: no information about Connection.");
                return Ok(self.state());
            }
        };

        let messages = agent_info.get_messages()?;

        if let Some((uid, message)) = self.verifier_sm.find_message_to_handle(messages) {
            self.handle_message(message.into())?;
            agent_info.update_message_status(uid, None)?;
        };

        let state = self.state();

        trace!("Verifier::update_state <<< state: {:?}", state);
        Ok(state)
    }

    pub fn update_state_with_message(&mut self, message: &str) -> VcxResult<u32> {
        trace!("Verifier::update_state_with_message >>> message: {:?}", secret!(message));
        debug!("Verifier {}: Updating state with message", self.get_source_id());

        let message: A2AMessage = ::serde_json::from_str(&message)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                              format!("Cannot updated Prover state with agent: Message deserialization failed with: {:?}", err)))?;

        self.handle_message(message.into())?;

        let state = self.state();

        trace!("Verifier::update_state_with_message <<< state: {:?}", state);
        Ok(state)
    }

    pub fn handle_message(&mut self, message: VerifierMessages) -> VcxResult<()> {
        trace!("Verifier::handle_message >>> message: {:?}", message);
        self.step(message)
    }

    pub fn verify_presentation(&mut self, presentation: Presentation) -> VcxResult<()> {
        trace!("Verifier::verify_presentation >>> presentation: {:?}", secret!(presentation));
        debug!("Verifier {}: Verifying presentation", self.get_source_id());
        self.step(VerifierMessages::PresentationReceived(presentation))
    }

    pub fn send_presentation_request(&mut self, connection_handle: Handle<Connections>) -> VcxResult<()> {
        trace!("Verifier::send_presentation_request >>> connection_handle: {:?}", connection_handle);
        debug!("Verifier {}: Sending presentation request", self.get_source_id());
        self.step(VerifierMessages::SendPresentationRequest(connection_handle))
    }

    pub fn request_proof(&mut self,
                         connection_handle: Handle<Connections>,
                         requested_attrs: String,
                         requested_predicates: String,
                         revocation_details: String,
                         name: String) -> VcxResult<()> {
        trace!("Verifier::request_proof >>> connection_handle: {:?}, requested_attrs: {:?}, requested_predicates: {:?}, revocation_details: {:?}, name: {:?}",
               connection_handle, secret!(requested_attrs), secret!(requested_predicates), secret!(revocation_details), secret!(name));
        debug!("Verifier {}: Requesting presentation", self.get_source_id());

        let presentation_request =
            ProofRequest::create()
                .set_name(name)
                .set_requested_attributes(requested_attrs)?
                .set_requested_predicates(requested_predicates)?
                .set_not_revoked_interval(revocation_details)?
                .set_nonce()?;
        self.step(VerifierMessages::RequestPresentation(connection_handle, presentation_request))
    }

    pub fn get_presentation_request(&self) -> VcxResult<&PresentationRequest> {
        trace!("Verifier::get_presentation_request >>>");
        debug!("Verifier {}: Getting presentation request", self.get_source_id());
        self.verifier_sm.presentation_request()
    }

    pub fn get_presentation_request_attach(&self) -> VcxResult<String> {
        trace!("Verifier::get_presentation_request_attach >>>");
        debug!("Verifier {}: Getting presentation request in Aries format", self.get_source_id());

        let proof_request = self.verifier_sm.presentation_request()?;

        ::serde_json::to_string(&proof_request)
            .map_err(|err| VcxError::from_msg(VcxErrorKind::SerializationError,
                                              format!("Cannot serialize ProofMessage. Err: {:?}", err)))
    }

    pub fn generate_presentation_request(&mut self) -> VcxResult<()> {
        trace!("Verifier::generate_presentation_request >>>");
        debug!("Verifier {}: Generating presentation request", self.get_source_id());

        self.step(VerifierMessages::PreparePresentationRequest())
    }

    pub fn set_connection(&mut self, connection_handle: Handle<Connections>) -> VcxResult<()> {
        debug!("Issuer {}: Sending credential", self.get_source_id());
        self.step(VerifierMessages::SetConnection(connection_handle))
    }


    pub fn get_presentation_proposal_request(&self) -> VcxResult<String> {
        trace!("Verifier::get_proposal_request >>>");
        debug!("Verifier {}: Getting presentation proposal request", self.get_source_id());
        self.verifier_sm.presentation_proposal()?.presentation_preview()
    }

    pub fn get_presentation(&self) -> VcxResult<&Presentation> {
        trace!("Verifier::get_presentation >>>");
        debug!("Verifier {}: Getting presentation", self.get_source_id());
        self.verifier_sm.presentation()
    }

    pub fn get_problem_report_message(&self) -> VcxResult<String> {
        trace!("Verifier::get_problem_report_message >>>");
        debug!("Verifier {}: Getting problem report message", self.get_source_id());

        let problem_report: Option<&ProblemReport> = self.verifier_sm.problem_report();
        Ok(json!(&problem_report).to_string())
    }

    pub fn step(&mut self, message: VerifierMessages) -> VcxResult<()> {
        self.verifier_sm = self.verifier_sm.clone().step(message)?;
        Ok(())
    }
}
