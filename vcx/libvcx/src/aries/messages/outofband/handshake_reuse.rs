use crate::aries::messages::outofband::v10::handshake_reuse::HandshakeReuse as HandshakeReuseV10;
use crate::aries::messages::outofband::v11::handshake_reuse::HandshakeReuse as HandshakeReuseV11;
use crate::aries::messages::thread::Thread;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(untagged)]
pub enum HandshakeReuse {
    V10(HandshakeReuseV10),
    V11(HandshakeReuseV11),
}

impl HandshakeReuse  {
    pub fn id(&self) -> String {
        match self {
            HandshakeReuse::V10(handshake_reuse) => handshake_reuse.id.to_string(),
            HandshakeReuse::V11(handshake_reuse) => handshake_reuse.id.to_string(),
        }
    }

    pub fn thread(&self) -> &Thread {
        match self {
            HandshakeReuse::V10(handshake_reuse) => &handshake_reuse.thread,
            HandshakeReuse::V11(handshake_reuse) => &handshake_reuse.thread,
        }
    }
}
