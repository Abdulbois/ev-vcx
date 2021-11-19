pub mod issuer_fsm;
pub mod states;
pub mod messages;

use self::{
    issuer_fsm::IssuerSM,
    messages::IssuerMessages,
};
use crate::error::prelude::*;
use crate::aries::messages::{
    a2a::A2AMessage,
    issuance::credential_offer::CredentialOffer,
    error::ProblemReport,
};

use crate::connection::Connections;
use crate::credential_def::CredentialDef;
use crate::utils::object_cache::Handle;

// Issuer

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Issuer {
    issuer_sm: IssuerSM
}

impl Issuer {
    pub fn create(cred_def_handle: Handle<CredentialDef>, credential_data: &str, source_id: &str, credential_name: &str) -> VcxResult<Issuer> {
        trace!("Issuer::issuer_create_credential >>> cred_def_handle: {:?}, credential_data: {:?}, source_id: {:?}",
               cred_def_handle, secret!(credential_data), source_id);
        debug!("Issuer {}: Creating credential Issuer state object", source_id);

        let cred_def_id = cred_def_handle.get_cred_def_id()?;
        let rev_reg_id = cred_def_handle.get_rev_reg_id()?;
        let tails_file = cred_def_handle.get_tails_file()?;
        let issuer_sm = IssuerSM::new(&cred_def_id, credential_data, rev_reg_id, tails_file, source_id, credential_name);
        Ok(Issuer { issuer_sm })
    }

    pub fn create_from_data(cred_def_id: &str, rev_reg_id: Option<String>, tails_file: Option<String>, credential_data: &str, source_id: &str, credential_name: &str) -> VcxResult<Issuer> {
        trace!("Issuer::issuer_create_credential >>> cred_def_id: {:?}, rev_reg_id: {:?}, tails_file {:?},  credential_data: {:?}, source_id: {:?}",
               cred_def_id, rev_reg_id, tails_file, secret!(credential_data), source_id);
        debug!("Issuer {}: Creating credential Issuer state object", source_id);

        let issuer_sm = IssuerSM::new(cred_def_id, credential_data, rev_reg_id, tails_file, source_id, credential_name);
        Ok(Issuer { issuer_sm })
    }

    pub fn send_credential_offer(&mut self, connection_handle: Handle<Connections>) -> VcxResult<()> {
        debug!("Issuer {}: Sending credential offer", self.get_source_id()?);
        self.step(IssuerMessages::CredentialInit(connection_handle))
    }

    pub fn send_credential(&mut self, connection_handle: Handle<Connections>) -> VcxResult<()> {
        debug!("Issuer {}: Sending credential", self.get_source_id()?);
        self.step(IssuerMessages::CredentialSend(connection_handle))
    }

    pub fn get_state(&self) -> VcxResult<u32> {
        Ok(self.issuer_sm.state())
    }

    pub fn get_source_id(&self) -> VcxResult<String> {
        Ok(self.issuer_sm.get_source_id())
    }

    pub fn get_credential_offer(&self) -> VcxResult<CredentialOffer> {
        self.issuer_sm.get_credential_offer()
            .ok_or(VcxError::from_msg(VcxErrorKind::InvalidState, format!("Invalid {} Issuer object state: `offer` not found", self.get_source_id()?)))
    }

    pub fn get_problem_report_message(&self) -> VcxResult<String> {
        trace!("Issuer::get_problem_report_message >>>");
        debug!("Issuer {}: Getting problem report message", self.issuer_sm.get_source_id());

        let problem_report: Option<&ProblemReport> = self.issuer_sm.problem_report();
        Ok(json!(&problem_report).to_string())
    }

    pub fn update_status(&mut self, msg: Option<String>) -> VcxResult<u32> {
        trace!("Issuer {}: update_state >>> msg: {:?}", self.get_source_id()?, secret!(msg));
        debug!("Issuer {}: updating state", self.get_source_id()?);

        match msg {
            Some(msg) => {
                let message: A2AMessage = ::serde_json::from_str(&msg)
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                                      format!("Cannot updated Issuer state with agent: Message deserialization failed with: {:?}", err)))?;

                self.step(message.into())?;
            }
            None => {
                self.issuer_sm = self.issuer_sm.clone().update_state()?;
            }
        };

        let state = self.get_state()?;

        trace!("Issuer::update_state <<< state: {:?}", state);
        Ok(state)
    }

    pub fn step(&mut self, message: IssuerMessages) -> VcxResult<()> {
        self.issuer_sm = self.issuer_sm.clone().handle_message(message)?;
        Ok(())
    }
}
