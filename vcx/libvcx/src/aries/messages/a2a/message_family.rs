use crate::aries::messages::a2a::protocol_registry::Actors;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, EnumIter)]
pub enum MessageTypeFamilies {
    Routing,
    Connections,
    Notification,
    Signature,
    CredentialIssuance,
    ReportProblem,
    PresentProof,
    TrustPing,
    DiscoveryFeatures,
    Basicmessage,
    Outofband,
    QuestionAnswer,
    Committedanswer,
    InviteAction,
    Unknown(String)
}

impl MessageTypeFamilies {
    pub const DID: &'static str = "did:sov:BzCbsNYhMrjHiqZDTUASHg;spec";
    pub const ENDPOINT: &'static str = "https://didcomm.org";

    pub fn version(&self) -> &'static str {
        match self {
            MessageTypeFamilies::Routing => "1.0",
            MessageTypeFamilies::Connections => "1.0",
            MessageTypeFamilies::Notification => "1.0",
            MessageTypeFamilies::Signature => "1.0",
            MessageTypeFamilies::CredentialIssuance => "1.0",
            MessageTypeFamilies::ReportProblem => "1.0",
            MessageTypeFamilies::PresentProof => "1.0",
            MessageTypeFamilies::TrustPing => "1.0",
            MessageTypeFamilies::DiscoveryFeatures => "1.0",
            MessageTypeFamilies::Basicmessage => "1.0",
            MessageTypeFamilies::Outofband => "1.0",
            MessageTypeFamilies::QuestionAnswer => "1.0",
            MessageTypeFamilies::Committedanswer => "1.0",
            MessageTypeFamilies::InviteAction => "0.9",
            MessageTypeFamilies::Unknown(_) => "1.0"
        }
    }

    pub fn id(&self) -> String {
        match self {
            MessageTypeFamilies::Routing |
            MessageTypeFamilies::Connections |
            MessageTypeFamilies::Notification |
            MessageTypeFamilies::Signature |
            MessageTypeFamilies::CredentialIssuance |
            MessageTypeFamilies::ReportProblem |
            MessageTypeFamilies::PresentProof |
            MessageTypeFamilies::TrustPing |
            MessageTypeFamilies::DiscoveryFeatures |
            MessageTypeFamilies::Basicmessage |
            MessageTypeFamilies::QuestionAnswer |
            MessageTypeFamilies::Committedanswer |
            MessageTypeFamilies::Unknown(_) => format!("{}/{}/{}", Self::DID, self.to_string(), self.version().to_string()),
            MessageTypeFamilies::Outofband |
            MessageTypeFamilies::InviteAction => format!("{}/{}/{}", Self::ENDPOINT, self.to_string(), self.version().to_string()),
        }
    }

    pub fn actors(&self) -> Option<(Option<Actors>, Option<Actors>)> {
        match self {
            MessageTypeFamilies::Routing => None,
            MessageTypeFamilies::Connections => Some((Some(Actors::Inviter), Some(Actors::Invitee))),
            MessageTypeFamilies::Notification => None,
            MessageTypeFamilies::Signature => None,
            MessageTypeFamilies::CredentialIssuance => Some((Some(Actors::Issuer), Some(Actors::Holder))),
            MessageTypeFamilies::ReportProblem => None,
            MessageTypeFamilies::PresentProof => Some((Some(Actors::Prover), Some(Actors::Verifier))),
            MessageTypeFamilies::TrustPing => Some((Some(Actors::Sender), Some(Actors::Receiver))),
            MessageTypeFamilies::DiscoveryFeatures => Some((Some(Actors::Sender), Some(Actors::Receiver))),
            MessageTypeFamilies::Basicmessage => Some((Some(Actors::Sender), Some(Actors::Receiver))),
            MessageTypeFamilies::Outofband => Some((None, Some(Actors::Receiver))),
            MessageTypeFamilies::QuestionAnswer => Some((None, Some(Actors::Receiver))),
            MessageTypeFamilies::Committedanswer => Some((None, Some(Actors::Receiver))),
            MessageTypeFamilies::InviteAction => Some((Some(Actors::Inviter), Some(Actors::Invitee))),
            MessageTypeFamilies::Unknown(_) => None
        }
    }
}

impl From<String> for MessageTypeFamilies {
    fn from(family: String) -> Self {
        match family.as_str() {
            "routing" => MessageTypeFamilies::Routing,
            "connections" => MessageTypeFamilies::Connections,
            "signature" => MessageTypeFamilies::Signature,
            "notification" => MessageTypeFamilies::Notification,
            "issue-credential" => MessageTypeFamilies::CredentialIssuance,
            "report-problem" => MessageTypeFamilies::ReportProblem,
            "present-proof" => MessageTypeFamilies::PresentProof,
            "trust_ping" => MessageTypeFamilies::TrustPing,
            "discover-features" => MessageTypeFamilies::DiscoveryFeatures,
            "basicmessage" => MessageTypeFamilies::Basicmessage,
            "out-of-band" => MessageTypeFamilies::Outofband,
            "questionanswer" => MessageTypeFamilies::QuestionAnswer,
            "committedanswer" => MessageTypeFamilies::Committedanswer,
            "invite-action" => MessageTypeFamilies::InviteAction,
            _ => MessageTypeFamilies::Unknown(family)
        }
    }
}

impl ToString for MessageTypeFamilies {
    fn to_string(&self) -> String {
        match self {
            MessageTypeFamilies::Routing => "routing".to_string(),
            MessageTypeFamilies::Connections => "connections".to_string(),
            MessageTypeFamilies::Notification => "notification".to_string(),
            MessageTypeFamilies::Signature => "signature".to_string(),
            MessageTypeFamilies::CredentialIssuance => "issue-credential".to_string(),
            MessageTypeFamilies::ReportProblem => "report-problem".to_string(),
            MessageTypeFamilies::PresentProof => "present-proof".to_string(),
            MessageTypeFamilies::TrustPing => "trust_ping".to_string(),
            MessageTypeFamilies::DiscoveryFeatures => "discover-features".to_string(),
            MessageTypeFamilies::Basicmessage => "basicmessage".to_string(),
            MessageTypeFamilies::Outofband => "out-of-band".to_string(),
            MessageTypeFamilies::QuestionAnswer => "questionanswer".to_string(),
            MessageTypeFamilies::Committedanswer => "committedanswer".to_string(),
            MessageTypeFamilies::InviteAction => "invite-action".to_string(),
            MessageTypeFamilies::Unknown(family) => family.to_string()
        }
    }
}

impl Default for MessageTypeFamilies {
    fn default() -> MessageTypeFamilies {
        MessageTypeFamilies::Unknown(String::new())
    }
}
