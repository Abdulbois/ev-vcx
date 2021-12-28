pub mod v10;
pub mod v20;

pub mod credential;
pub mod credential_offer;
pub mod credential_proposal;
pub mod credential_request;
pub mod credential_ack;
pub mod credential_preview;

#[cfg(test)]
pub mod test {
    use crate::aries::messages::ack;
    use crate::aries::messages::error;
    use super::v10::credential_offer::tests::_credential_offer;

    pub fn _ack() -> ack::Ack {
        ack::tests::_ack().set_thread_id(&_credential_offer().id.0)
    }

    pub fn _problem_report() -> error::ProblemReport {
        error::tests::_problem_report().set_thread_id(&_credential_offer().id.0)
    }
}
