use serde_json;

use crate::error::prelude::*;
use crate::aries::messages::thread::Thread;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
pub struct MessageWithThread {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "~thread")]
    pub thread: Option<Thread>,
}

pub fn extract_thread_id(message: &str) -> VcxResult<String> {
    trace!("Message::extract_thread_id >>>");
    debug!("Message::extracting thread id for message");

    let message_with_thread: MessageWithThread = serde_json::from_str(message)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::InvalidJson,
            format!("Unable to parse MessageWithThread from JSON string. Err: {:?}", err),
        ))?;

    let thread_id = message_with_thread.thread.as_ref()
        .and_then(|thread| thread.thid.clone())
        .unwrap_or_else(|| message_with_thread.id.to_string());

    trace!("Message::vcx_extract_thread_id <<< thread_id: {:?}", thread_id);
    Ok(thread_id)
}


#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn get_thread_id_for_message_withot_thread() {
        let message = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential"}"#;
        let thread_id = extract_thread_id(message).unwrap();
        let expected_thread_id = "testid";
        assert_eq!(expected_thread_id, thread_id);
    }

    #[test]
    fn get_thread_id_for_message_with_thread() {
        let message = r#"{"@id":"testid","@type":"did:sov:BzCbsNYhMrjHiqZDTUASHg;spec/issue-credential/1.0/offer-credential","~thread":{"thid":"internal_testid"}}"#;
        let thread_id = extract_thread_id(message).unwrap();
        let expected_thread_id = "internal_testid";
        assert_eq!(expected_thread_id, thread_id);
    }
}