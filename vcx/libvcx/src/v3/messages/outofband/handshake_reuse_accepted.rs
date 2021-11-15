use crate::v3::messages::outofband::v10::handshake_reuse_accepted::HandshakeReuseAccepted as HandshakeReuseAcceptedV10;
use crate::v3::messages::outofband::v11::handshake_reuse_accepted::HandshakeReuseAccepted as HandshakeReuseAcceptedV11;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum HandshakeReuseAccepted {
    V10(HandshakeReuseAcceptedV10),
    V11(HandshakeReuseAcceptedV11),
}