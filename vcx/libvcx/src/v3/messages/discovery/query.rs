use crate::v3::messages::a2a::{MessageId, A2AMessage};
use crate::v3::messages::a2a::message_type::{
    MessageType,
    MessageTypePrefix,
    MessageTypeVersion,
};
use crate::v3::messages::a2a::message_family::MessageTypeFamilies;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Query {
    #[serde(rename = "@id")]
    pub id: MessageId,
    #[serde(rename = "@type")]
    pub type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>
}

impl Query {
    pub fn create() -> Query {
        Query::default()
    }

    pub fn set_query(mut self, query: Option<String>) -> Self {
        self.query = query;
        self
    }

    pub fn set_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }
}

impl Default for Query {
    fn default() -> Query {
        Query {
            id: MessageId::default(),
            type_: MessageType {
                prefix: MessageTypePrefix::DID,
                family: MessageTypeFamilies::DiscoveryFeatures,
                version: MessageTypeVersion::V10,
                type_: A2AMessage::QUERY.to_string()
            },
            query: Default::default(),
            comment: Default::default(),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn _query_string() -> String {
        String::from("did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/")
    }


    fn _comment() -> String {
        String::from("I'm wondering if we can...")
    }

    pub fn _query() -> Query {
        Query {
            id: MessageId::id(),
            query: Some(_query_string()),
            comment: Some(_comment()),
            ..Query::default()
        }
    }

    #[test]
    fn test_query_build_works() {
        let query: Query = Query::default()
            .set_query(Some(_query_string()))
            .set_comment(Some(_comment()));

        assert_eq!(_query(), query);
        let expected = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/discover-features/1.0/query","comment":"I'm wondering if we can...","query":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/"}"#;
        assert_eq!(expected, json!(query).to_string());
    }
}