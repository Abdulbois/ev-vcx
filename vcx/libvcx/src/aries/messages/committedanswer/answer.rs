use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::thread::Thread;
use crate::aries::messages::committedanswer::question::{Question, QuestionResponse};
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;
use crate::error::VcxResult;
use crate::utils::libindy::crypto;
#[cfg(any(not(test)))]
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Answer {
    #[serde(rename = "@id")]
    pub id: Option<MessageId>,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(rename = "response.@sig")]
    pub signature: ResponseSignature,
    #[serde(rename = "~thread")]
    pub thread: Thread,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseSignature {
    pub signature: String,
    pub sig_data: String,
    pub timestamp: String,
}

impl Answer {
    pub fn create() -> Answer {
        Answer::default()
    }

    pub fn sign(mut self, question: &Question, response: &QuestionResponse, key: &str) -> VcxResult<Self> {
        trace!("Answer::sign >>> question: {:?}", secret!(question));

        let sig_data = base64::encode(&response.nonce);

        let signature = crypto::sign(key, sig_data.as_bytes())?;

        let signature = base64::encode(&signature);

        self.signature = ResponseSignature {
            signature,
            sig_data,
            ..Default::default()
        };

        trace!("Answer::sign <<<");
        Ok(self)
    }

    pub fn set_signature(mut self, signature: ResponseSignature) -> Self {
        self.signature = signature;
        self
    }
}

impl Default for Answer {
    fn default() -> Answer {
        Answer {
            id: Some(MessageId::default()),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::Committedanswer,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::ANSWER.to_string()
            },
            signature: Default::default(),
            thread: Default::default(),
        }
    }
}

impl Default for ResponseSignature {
    #[cfg(all(test))]
    fn default() -> ResponseSignature {
        ResponseSignature {
            signature: Default::default(),
            sig_data: Default::default(),
            timestamp: "111".to_string()
        }
    }

    #[cfg(any(not(test)))]
    fn default() -> ResponseSignature {
        ResponseSignature {
            signature: Default::default(),
            sig_data: Default::default(),
            timestamp: Utc::now().timestamp().to_string()
        }
    }
}

threadlike!(Answer);

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::aries::messages::connection::response::tests::*;

    fn _answer_text() -> String {
        String::from("Yes, it's me".to_string())
    }

    fn _time() -> String {
        String::from("2018-12-13T17:29:34+0000".to_string())
    }

    fn _answer() -> Answer {
        Answer {
            id: Some(MessageId::default()),
            thread: _thread(),
            signature: Default::default(),
            ..Answer::default()
        }
    }

    #[test]
    fn test_answer_message_build_works() {
        let answer: Answer = Answer::default()
            .set_thread(_thread());

        assert_eq!(_answer(), answer);

        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/committedanswer/1.0/answer","response.@sig":{"sig_data":"","signature":"","timestamp":"111"},"~thread":{"received_orders":{},"sender_order":0,"thid":"test_id"}}"#;
        assert_eq!(expected, json!(answer).to_string());
    }
}
