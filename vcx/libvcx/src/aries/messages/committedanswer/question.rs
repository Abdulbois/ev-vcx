use crate::aries::messages::a2a::{MessageId, A2AMessage};
use crate::aries::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::aries::messages::a2a::message_family::MessageTypeFamilies;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Question {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    pub question_text: String,
    pub question_detail: Option<String>,
    #[serde(default)]
    pub external_links: Vec<::serde_json::Value>,
    pub valid_responses: Vec<QuestionResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct QuestionResponse {
    pub text: String,
    pub nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_msg_id: Option<String>,
}

impl Question {
    pub fn create() -> Question {
        Question::default()
    }

    pub fn set_question_text(mut self, question_text: String) -> Self {
        self.question_text = question_text;
        self
    }

    pub fn set_question_detail(mut self, question_detail: Option<String>) -> Self {
        self.question_detail = question_detail;
        self
    }

    pub fn set_external_links(mut self, external_links: Vec<serde_json::Value>) -> Self {
        self.external_links = external_links;
        self
    }

    pub fn set_valid_responses(mut self, valid_responses: Vec<QuestionResponse>) -> Self {
        self.valid_responses = valid_responses;
        self
    }
}

impl Default for Question {
    fn default() -> Question {
        Question {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::Committedanswer,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::QUESTION.to_string()
            },
            question_text: Default::default(),
            question_detail: Default::default(),
            external_links: Default::default(),
            valid_responses: Default::default()
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn _text() -> String {
        String::from("Alice, are you on the phone with Bob from Faber Bank right now?")
    }

    fn _detail() -> String {
        String::from("This is optional fine-print giving context to the question and its various answers.")
    }

    fn _nonce() -> String {
        String::from("1000000")
    }

    fn _relationship() -> String {
        String::from("2cC2FpqAu2P2XccsvKk7w1")
    }

    fn _valid_responses() -> Vec<QuestionResponse> {
        vec![
            QuestionResponse {
                text: "Yes, it's me".to_string(),
                nonce: "n1".to_string(),
                ref_msg_id: None
            },
            QuestionResponse {
                text: "No, that's not me!".to_string(),
                nonce: "n1".to_string(),
                ref_msg_id: None
            },
        ]
    }

    pub fn _question() -> Question {
        Question {
            id: MessageId::id(),
            question_text: _text(),
            question_detail: Some(_detail()),
            valid_responses: _valid_responses(),
            external_links: vec![],
            ..Question::default()
        }
    }

    #[test]
    fn test_question_message_build_works() {
        let question: Question = Question::default()
            .set_question_text(_text())
            .set_question_detail(Some(_detail()))
            .set_valid_responses(_valid_responses());
        assert_eq!(_question(), question);

        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/committedanswer/1.0/question","external_links":[],"question_detail":"This is optional fine-print giving context to the question and its various answers.","question_text":"Alice, are you on the phone with Bob from Faber Bank right now?","valid_responses":[{"nonce":"n1","text":"Yes, it's me"},{"nonce":"n1","text":"No, that's not me!"}]}"#;
        assert_eq!(expected, json!(question).to_string());
    }
}
