use crate::settings;
use crate::agent::messages::message_type::{MessageTypes, MessageTypeV2};
use crate::agent::messages::payload::{Payloads, PayloadTypes, PayloadKinds, PayloadV1};
use crate::utils::{httpclient, constants};
use crate::error::prelude::*;
use crate::settings::protocol::ProtocolTypes;
use crate::utils::httpclient::AgencyMock;
use crate::legacy::messages::issuance::credential_offer::{set_cred_offer_ref_message, CredentialOffer};
use crate::legacy::messages::proof_presentation::proof_request::{set_proof_req_ref_message, ProofRequestMessage};
use crate::legacy::messages::issuance::credential_request::set_cred_req_ref_message;
use crate::aries::messages::a2a::A2AMessage as AriesA2AMessage;
use crate::aries::utils::encryption_envelope::EncryptionEnvelope;
use std::convert::TryInto;
use crate::agent::messages::{A2AMessage, parse_response_from_agency, A2AMessageV1, A2AMessageV2, A2AMessageKinds, MessageStatusCode, prepare_message_for_agency, GeneralMessage, prepare_message_for_agent, RemoteMessageType, i8_as_u8_slice, get_messages};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetMessages {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "excludePayload")]
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    uids: Option<Vec<String>>,
    #[serde(rename = "statusCodes")]
    #[serde(skip_serializing_if = "Option::is_none")]
    status_codes: Option<Vec<MessageStatusCode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "pairwiseDIDs")]
    pairwise_dids: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetMessagesResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    msgs: Vec<Message>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MessagesByConnections {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "msgsByConns")]
    #[serde(default)]
    msgs: Vec<MessageByConnection>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct MessageByConnection {
    #[serde(rename = "pairwiseDID")]
    pub pairwise_did: String,
    pub msgs: Vec<Message>,
}

#[derive(Debug)]
pub struct GetMessagesBuilder {
    to_did: String,
    to_vk: String,
    agent_did: String,
    agent_vk: String,
    exclude_payload: Option<String>,
    uids: Option<Vec<String>>,
    status_codes: Option<Vec<MessageStatusCode>>,
    pairwise_dids: Option<Vec<String>>,
    version: ProtocolTypes,
}

impl GetMessagesBuilder {
    pub fn create() -> GetMessagesBuilder {
        trace!("GetMessages::create_message >>>");

        GetMessagesBuilder {
            to_did: String::new(),
            to_vk: String::new(),
            agent_did: String::new(),
            agent_vk: String::new(),
            uids: None,
            exclude_payload: None,
            status_codes: None,
            pairwise_dids: None,
            version: settings::get_protocol_type(),
        }
    }

    #[cfg(test)]
    pub fn create_v1() -> GetMessagesBuilder {
        let mut builder = GetMessagesBuilder::create();
        builder.version = ProtocolTypes::V1;
        builder
    }

    pub fn uid(&mut self, uids: Option<Vec<String>>) -> VcxResult<&mut Self> {
        //Todo: validate msg_uid??
        self.uids = uids;
        Ok(self)
    }

    pub fn status_codes(&mut self, status_codes: Option<Vec<MessageStatusCode>>) -> VcxResult<&mut Self> {
        self.status_codes = status_codes;
        Ok(self)
    }

    pub fn pairwise_dids(&mut self, pairwise_dids: Option<Vec<String>>) -> VcxResult<&mut Self> {
        //Todo: validate msg_uid??
        self.pairwise_dids = pairwise_dids;
        Ok(self)
    }

    pub fn include_edge_payload(&mut self, payload: &str) -> VcxResult<&mut Self> {
        //todo: is this a json value, String??
        self.exclude_payload = Some(payload.to_string());
        Ok(self)
    }

    pub fn version(&mut self, version: &Option<ProtocolTypes>) -> VcxResult<&mut Self> {
        self.version = match version {
            Some(version) => version.clone(),
            None => settings::get_protocol_type()
        };
        Ok(self)
    }

    pub fn send_secure(&mut self) -> VcxResult<Vec<Message>> {
        trace!("GetMessagesBuilder::send_secure >>>");

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        if settings::agency_mocks_enabled() && response.is_empty() {
            return Ok(Vec::new());
        }

        self.parse_response(&response)
    }

    fn parse_response(&self, response: &[u8]) -> VcxResult<Vec<Message>> {
        trace!("GetMessagesBuilder::parse_response >>>");

        let mut response = parse_response_from_agency(response, &self.version)?;

        match response.swap_remove(0) {
            A2AMessage::Version1(A2AMessageV1::GetMessagesResponse(res)) => Ok(res.msgs),
            A2AMessage::Version2(A2AMessageV2::GetMessagesResponse(res)) => Ok(res.msgs),
            _ => Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response does not match any variant of GetMessagesResponse"))
        }
    }

    pub fn download_messages(&mut self) -> VcxResult<Vec<MessageByConnection>> {
        trace!("GetMessagesBuilder::download >>>");

        let data = self.prepare_download_request()?;

        let response = httpclient::post_u8(&data)?;

        if settings::agency_mocks_enabled() && response.len() == 0 {
            return Ok(Vec::new());
        }

        let response = self.parse_download_messages_response(response)?;

        Ok(response)
    }

    fn prepare_download_request(&self) -> VcxResult<Vec<u8>> {
        trace!("GetMessagesBuilder::prepare_download_request >>>");

        let message = match self.version {
            ProtocolTypes::V1 =>
                A2AMessage::Version1(
                    A2AMessageV1::GetMessages(
                        GetMessages {
                            msg_type: MessageTypes::MessageTypeV1(MessageTypes::build_v1(A2AMessageKinds::GetMessagesByConnections)),
                            exclude_payload: self.exclude_payload.clone(),
                            uids: self.uids.clone(),
                            status_codes: self.status_codes.clone(),
                            pairwise_dids: self.pairwise_dids.clone(),
                        }
                    )
                ),
            ProtocolTypes::V2 |
            ProtocolTypes::V3 |
            ProtocolTypes::V4 =>
                A2AMessage::Version2(
                    A2AMessageV2::GetMessages(
                        GetMessages {
                            msg_type: MessageTypes::MessageTypeV2(MessageTypes::build_v2(A2AMessageKinds::GetMessagesByConnections)),
                            exclude_payload: self.exclude_payload.clone(),
                            uids: self.uids.clone(),
                            status_codes: self.status_codes.clone(),
                            pairwise_dids: self.pairwise_dids.clone(),
                        }
                    )
                ),
        };

        trace!("GetMessagesBuilder::prepare_request >>> message: {:?}", secret!(message));

        let agency_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

        prepare_message_for_agency(&message, &agency_did, &self.version)
    }

    fn parse_download_messages_response(&self, response: Vec<u8>) -> VcxResult<Vec<MessageByConnection>> {
        trace!("GetMessagesBuilder::parse_download_messages_response >>>");
        let mut response = parse_response_from_agency(&response, &self.version)?;

        trace!("parse_download_messages_response: parsed response {:?}", response);
        let msgs = match response.swap_remove(0) {
            A2AMessage::Version1(A2AMessageV1::GetMessagesByConnectionsResponse(res)) => res.msgs,
            A2AMessage::Version2(A2AMessageV2::GetMessagesByConnectionsResponse(res)) => res.msgs,
            _ => return Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response does not match any variant of GetMessagesByConnectionsResponse"))
        };

        msgs
            .into_iter()
            .map(|connection| {
                crate::utils::libindy::crypto::get_local_verkey(&connection.pairwise_did)
                    .map(|vk| {
                        let msgs = connection.msgs.iter().map(|message| message.decrypt(&vk)).collect();
                        MessageByConnection {
                            pairwise_did: connection.pairwise_did,
                            msgs,
                        }
                    })
            })
            .collect()
    }
}

//Todo: Every GeneralMessage extension, duplicates code
impl GeneralMessage for GetMessagesBuilder {
    type Msg = GetMessagesBuilder;

    fn set_agent_did(&mut self, did: String) { self.agent_did = did; }
    fn set_agent_vk(&mut self, vk: String) { self.agent_vk = vk; }
    fn set_to_did(&mut self, to_did: String) { self.to_did = to_did; }
    fn set_to_vk(&mut self, to_vk: String) { self.to_vk = to_vk; }

    fn prepare_request(&mut self) -> VcxResult<Vec<u8>> {
        trace!("GetMessagesBuilder::prepare_request >>>");

        let message = match self.version {
            ProtocolTypes::V1 =>
                A2AMessage::Version1(
                    A2AMessageV1::GetMessages(
                        GetMessages {
                            msg_type: MessageTypes::MessageTypeV1(MessageTypes::build_v1(A2AMessageKinds::GetMessages)),
                            exclude_payload: self.exclude_payload.clone(),
                            uids: self.uids.clone(),
                            status_codes: self.status_codes.clone(),
                            pairwise_dids: self.pairwise_dids.clone(),
                        }
                    )
                ),
            ProtocolTypes::V2 |
            ProtocolTypes::V3 |
            ProtocolTypes::V4 =>
                A2AMessage::Version2(
                    A2AMessageV2::GetMessages(
                        GetMessages {
                            msg_type: MessageTypes::MessageTypeV2(MessageTypes::build_v2(A2AMessageKinds::GetMessages)),
                            exclude_payload: self.exclude_payload.clone(),
                            uids: self.uids.clone(),
                            status_codes: self.status_codes.clone(),
                            pairwise_dids: self.pairwise_dids.clone(),
                        }
                    )
                ),
        };

        trace!("GetMessagesBuilder::prepare_request >>> message: {:?}", secret!(message));

        prepare_message_for_agent(vec![message], &self.to_vk, &self.agent_did, &self.agent_vk, &self.version)
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryDetails {
    to: String,
    status_code: String,
    last_updated_date_time: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum MessagePayload {
    V1(Vec<i8>),
    V2(::serde_json::Value),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(rename = "statusCode")]
    pub status_code: MessageStatusCode,
    pub payload: Option<MessagePayload>,
    #[serde(rename = "senderDID")]
    pub sender_did: String,
    pub uid: String,
    #[serde(rename = "type")]
    pub msg_type: RemoteMessageType,
    pub ref_msg_id: Option<String>,
    #[serde(skip_deserializing)]
    pub delivery_details: Vec<DeliveryDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decrypted_payload: Option<String>,
}

impl Message {
    pub fn payload(&self) -> VcxResult<Vec<u8>> {
        trace!("Message::payload >>>");
        match &self.payload {
            Some(MessagePayload::V1(payload)) => Ok(i8_as_u8_slice(payload).to_vec()),
            Some(MessagePayload::V2(payload)) => serde_json::to_vec(payload)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot represent JSON object as bytes. Err: {:?}", err))),
            None => Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Agency response Message doesn't contain any payload")),
        }
    }

    pub fn decrypt(&self, vk: &str) -> Message {
        trace!("Message::decrypt >>> vk: {:?}", secret!(vk));

        // TODO: must be Result
        let mut new_message = self.clone();
        if let Some(ref payload) = self.payload {
            if settings::is_strict_aries_protocol_set() {
                if let Ok(decrypted_payload) = self._decrypt_v3_message() {
                    new_message.msg_type = RemoteMessageType::Other(String::from("aries"));
                    new_message.decrypted_payload = ::serde_json::to_string(&json!(decrypted_payload)).ok();
                    new_message.payload = None;
                    return new_message;
                }
            }

            let decrypted_payload = match payload {
                MessagePayload::V1(payload) => {
                    Payloads::decrypt_payload_v1(&vk, &payload)
                        .map(Payloads::PayloadV1)
                }
                MessagePayload::V2(payload) => {
                    Payloads::decrypt_payload_v2(&vk, &payload)
                        .map(Payloads::PayloadV2)
                }
            };

            if let Ok(mut decrypted_payload) = decrypted_payload {
                Self::_set_ref_msg_id(&mut decrypted_payload, &self.uid)
                    .map_err(|err| error!("Could not set ref_msg_id: {:?}", err)).ok();
                new_message.decrypted_payload = ::serde_json::to_string(&decrypted_payload).ok();
            } else if let Ok(decrypted_payload) = self._decrypt_v3_message() {
                new_message.msg_type = RemoteMessageType::Other(String::from("aries"));
                new_message.decrypted_payload = ::serde_json::to_string(&json!(decrypted_payload)).ok()
            } else {
                warn!("Message::decrypt <<< were not able to decrypt message, setting null");
                new_message.decrypted_payload = ::serde_json::to_string(&json!(null)).ok();
            }
        }
        new_message.payload = None;

        trace!("Message::decrypt <<< message: {:?}", secret!(new_message));

        new_message
    }

    fn _set_ref_msg_id(decrypted_payload: &mut Payloads, msg_id: &str) -> VcxResult<()> {
        trace!("_set_ref_msg_id >>> decrypted_payload: {:?}, msg_id: {:?}", secret!(decrypted_payload), msg_id);
        match decrypted_payload {
            Payloads::PayloadV1(ref mut payload) => {
                let type_ = payload.type_.name.as_str();
                trace!("_set_ref_msg_id >>> message type: {:?}", secret!(type_));

                match type_ {
                    "CRED_OFFER" | "credential-offer" => {
                        let offer = set_cred_offer_ref_message(&payload.msg, None, &msg_id)?;
                        payload.msg = json!(offer).to_string();
                    }
                    "CRED_REQ" | "credential-request" => {
                        let cred_req = set_cred_req_ref_message(&payload.msg, &msg_id)?;
                        payload.msg = json!(cred_req).to_string();
                    }
                    "PROOF_REQUEST" | "presentation-request" => {
                        let proof_request = set_proof_req_ref_message(&payload.msg, None, &msg_id)?;
                        payload.msg = json!(proof_request).to_string();
                    }
                    _ => {}
                }
            }
            Payloads::PayloadV2(ref mut payload) => {
                let message_type: MessageTypeV2 = serde_json::from_value(json!(payload.type_))
                    .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse message type: {:?}", err)))?;
                let type_ = message_type.type_.as_str();
                trace!("_set_ref_msg_id >>> message type: {:?}", secret!(type_));

                match type_ {
                    "credential-offer" => {
                        let offer = set_cred_offer_ref_message(&payload.msg, Some(payload.thread.clone()), &msg_id)?;
                        payload.msg = json!(offer).to_string();
                    }
                    "credential-request" => {
                        let cred_req = set_cred_req_ref_message(&payload.msg, &msg_id).unwrap();
                        payload.msg = json!(cred_req).to_string();
                    }
                    "presentation-request" => {
                        let proof_request = set_proof_req_ref_message(&payload.msg, Some(payload.thread.clone()), &msg_id)?;
                        payload.msg = json!(proof_request).to_string();
                    }
                    _ => {}
                }
            }
        };
        trace!("_set_ref_msg_id <<<");
        Ok(())
    }

    fn _decrypt_v3_message(&self) -> VcxResult<crate::agent::messages::payload::PayloadV1> {
        trace!("_decrypt_v3_message >>>");

        let a2a_message = EncryptionEnvelope::open(self.payload()?)?;

        let (kind, msg) = match a2a_message {
            AriesA2AMessage::PresentationRequest(presentation_request) => {
                if settings::is_strict_aries_protocol_set() {
                    (PayloadKinds::ProofRequest, json!(&presentation_request).to_string())
                } else {
                    let converted_message: ProofRequestMessage = presentation_request.try_into()?;
                    (PayloadKinds::ProofRequest, json!(&converted_message).to_string())
                }
            }
            AriesA2AMessage::CredentialOffer(offer) => {
                if settings::is_strict_aries_protocol_set() {
                    (PayloadKinds::CredOffer, json!(&offer).to_string())
                } else {
                    let cred_offer: CredentialOffer = offer.try_into()?;
                    (PayloadKinds::CredOffer, json!(vec![cred_offer]).to_string())
                }
            }
            AriesA2AMessage::Credential(credential) => {
                if settings::is_strict_aries_protocol_set() {
                    (PayloadKinds::Cred, json!(&credential).to_string())
                } else {
                    (PayloadKinds::Other(String::from("credential")), json!(&credential).to_string())
                }
            }
            AriesA2AMessage::PresentationProposal(presentation_proposal) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::PROPOSE_PRESENTATION)), json!(&presentation_proposal).to_string())
            }
            AriesA2AMessage::Presentation(presentation) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::PRESENTATION)), json!(&presentation).to_string())
            }
            AriesA2AMessage::Ping(ping) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::PING)), json!(&ping).to_string())
            }
            AriesA2AMessage::PingResponse(ping_response) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::PING_RESPONSE)), json!(&ping_response).to_string())
            }
            AriesA2AMessage::Query(query) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::QUERY)), json!(&query).to_string())
            }
            AriesA2AMessage::Disclose(disclose) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::DISCLOSE)), json!(&disclose).to_string())
            }
            AriesA2AMessage::HandshakeReuse(reuse) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::OUTOFBAND_HANDSHAKE_REUSE)), json!(&reuse).to_string())
            }
            AriesA2AMessage::HandshakeReuseAccepted(reuse) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::OUTOFBAND_HANDSHAKE_REUSE_ACCEPTED)), json!(&reuse).to_string())
            }
            AriesA2AMessage::Question(question) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::QUESTION)), json!(&question).to_string())
            }
            AriesA2AMessage::Answer(answer) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::ANSWER)), json!(&answer).to_string())
            }
            AriesA2AMessage::CommittedQuestion(question) => {
                (PayloadKinds::Other(String::from("committed-question")), json!(&question).to_string())
            }
            AriesA2AMessage::CommittedAnswer(answer) => {
                (PayloadKinds::Other(String::from("committed-answer")), json!(&answer).to_string())
            }
            AriesA2AMessage::BasicMessage(message) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::BASIC_MESSAGE)), json!(&message).to_string())
            }
            AriesA2AMessage::InviteForAction(invite) => {
                (PayloadKinds::Other(String::from("invite-action")), json!(&invite).to_string())
            }
            AriesA2AMessage::InviteForActionReject(reject) |
            AriesA2AMessage::CredentialReject(reject) |
            AriesA2AMessage::PresentationReject(reject) => {
                (PayloadKinds::Other(String::from(AriesA2AMessage::PROBLEM_REPORT)), json!(&reject).to_string())
            }
            msg => {
                (PayloadKinds::Other(String::from("aries")), json!(&msg).to_string())
            }
        };

        trace!("_decrypt_v3_message <<< kind: {:?}, msg: {:?}", secret!(kind), secret!(msg));

        let payload = PayloadV1 {
            type_: PayloadTypes::build_v1(kind, "json"),
            msg,
        };

        Ok(payload)
    }
}

pub fn get_connection_messages(pw_did: &str, pw_vk: &str, agent_did: &str, agent_vk: &str, msg_uid: Option<Vec<String>>, status_codes: Option<Vec<MessageStatusCode>>, version: &Option<ProtocolTypes>) -> VcxResult<Vec<Message>> {
    trace!("get_connection_messages >>> pw_did: {}, pw_vk: {}, agent_vk: {}, msg_uid: {:?}",
           secret!(pw_did), secret!(pw_vk), secret!(agent_vk), secret!(msg_uid));

    let response = get_messages()
        .to(&pw_did)?
        .to_vk(&pw_vk)?
        .agent_did(&agent_did)?
        .agent_vk(&agent_vk)?
        .uid(msg_uid)?
        .status_codes(status_codes)?
        .version(version)?
        .send_secure()
        .map_err(|err| err.extend("Cannot get agent"))?;

    trace!("message returned: {:?}", secret!(response));
    Ok(response)
}

pub fn get_ref_msg(msg_id: &str, pw_did: &str, pw_vk: &str, agent_did: &str, agent_vk: &str) -> VcxResult<(String, MessagePayload)> {
    trace!("get_ref_msg >>> msg_id: {}, pw_did: {}, pw_vk: {}, agent_did: {}, agent_vk: {}",
           msg_id, secret!(pw_did), secret!(pw_vk), secret!(agent_did), secret!(agent_vk));

    let message: Vec<Message> = get_connection_messages(pw_did, pw_vk, agent_did, agent_vk, Some(vec![msg_id.to_string()]), None, &None)?;
    trace!("checking for referent for message: {:?}", secret!(message));

    let msg_id = match message.get(0).as_ref().and_then(|message| message.ref_msg_id.as_ref()) {
        Some(ref ref_msg_id) if message[0].status_code == MessageStatusCode::Accepted => ref_msg_id.to_string(),
        _ => return Err(VcxError::from_msg(VcxErrorKind::NotReady, "Cannot find referent message")),
    };

    let message: Vec<Message> = get_connection_messages(pw_did, pw_vk, agent_did, agent_vk, Some(vec![msg_id]), None, &None)?;

    trace!("receivedpending agent: {:?}", secret!(message));

    // this will work for both credReq and proof types
    match message.get(0).as_ref().and_then(|message| message.payload.as_ref()) {
        Some(payload) if message[0].status_code == MessageStatusCode::Received => {
            // TODO: check returned verkey
            Ok((message[0].uid.clone(), payload.to_owned()))
        }
        _ => Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, "Cannot find referent message"))
    }
}

fn _parse_status_code(status_codes: Option<Vec<String>>) -> VcxResult<Option<Vec<MessageStatusCode>>> {
    match status_codes {
        Some(codes) => {
            let codes = codes
                .iter()
                .map(|code|
                    ::serde_json::from_str::<MessageStatusCode>(&format!("\"{}\"", code))
                        .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson, format!("Cannot parse message status code: {}", err)))
                ).collect::<VcxResult<Vec<MessageStatusCode>>>()?;
            Ok(Some(codes))
        }
        None => Ok(None)
    }
}

pub fn download_messages(pairwise_dids: Option<Vec<String>>, status_codes: Option<Vec<String>>, uids: Option<Vec<String>>) -> VcxResult<Vec<MessageByConnection>> {
    trace!("download_messages >>> pairwise_dids: {:?}, status_codes: {:?}, uids: {:?}",
           secret!(pairwise_dids), status_codes, uids);
    debug!("Agency: Downloading agent");

    AgencyMock::set_next_response(constants::GET_ALL_MESSAGES_RESPONSE);

    let status_codes = _parse_status_code(status_codes)?;

    let response =
        get_messages()
            .uid(uids)?
            .status_codes(status_codes)?
            .pairwise_dids(pairwise_dids)?
            .version(&Some(crate::settings::get_protocol_type()))?
            .download_messages()?;

    debug!("Agency: received agent: {:?}", secret!(response));
    trace!("download_messages <<< agent: {:?}", secret!(response));
    Ok(response)
}

pub fn download_agent_messages(status_codes: Option<Vec<String>>, uids: Option<Vec<String>>) -> VcxResult<Vec<Message>> {
    trace!("download_agent_messages >>> status_codes: {:?}, uids: {:?}", status_codes, secret!(uids));
    debug!("Agency: Downloading Agent agent");

    AgencyMock::set_next_response(constants::GET_ALL_MESSAGES_RESPONSE);

    let status_codes = _parse_status_code(status_codes)?;

    let response =
        get_messages()
            .to(&crate::settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_DID)?)?
            .to_vk(&crate::settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?)?
            .agent_did(&crate::settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?)?
            .agent_vk(&crate::settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY)?)?
            .uid(uids)?
            .status_codes(status_codes)?
            .send_secure()?;

    debug!("Agency: received Agent agent: {:?}", secret!(response));
    trace!("download_agent_messages <<< agent: {:?}", secret!(response));
    Ok(response)
}

pub fn download_message(uid: String) -> VcxResult<Message> {
    trace!("download_message >>> uid: {:?}", uid);
    debug!("Agency: Downloading message {:?}", uid);

    AgencyMock::set_next_response(constants::GET_ALL_MESSAGES_RESPONSE);

    let mut messages: Vec<Message> =
        get_messages()
            .uid(Some(vec![uid.clone()]))?
            .version(&Some(crate::settings::get_protocol_type()))?
            .download_messages()?
            .into_iter()
            .flat_map(|msgs_by_connection| msgs_by_connection.msgs)
            .collect();

    if messages.is_empty() {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse,
                                      format!("Message for the given uid:\"{}\" not found.", uid)));
    }

    if messages.len() > 1 {
        return Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse,
                                      format!("More than one message was received for the given uid:\"{}\". \
                                      Please, use `vcx_messages_download` function to retrieve several agent.", uid)));
    }

    let message = messages.remove(0);

    debug!("Agency: received message: {:?}", secret!(message));
    trace!("download_message <<< message: {:?}", secret!(message));
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::constants::{GET_MESSAGES_RESPONSE, GET_ALL_MESSAGES_RESPONSE};
    #[cfg(any(feature = "agency", feature = "pool_tests"))]
    use std::thread;
    #[cfg(any(feature = "agency", feature = "pool_tests"))]
    use std::time::Duration;
    use crate::utils::devsetup::*;

    #[test]
    fn test_parse_get_messages_response() {
        let _setup = SetupMocks::init();

        let result = GetMessagesBuilder::create_v1().parse_response(GET_MESSAGES_RESPONSE).unwrap();
        assert_eq!(result.len(), 3)
    }

    #[test]
    fn test_parse_get_connection_messages_response() {
        let _setup = SetupMocks::init();

        let result = GetMessagesBuilder::create().version(&Some(ProtocolTypes::V1)).unwrap().parse_download_messages_response(GET_ALL_MESSAGES_RESPONSE.to_vec()).unwrap();
        assert_eq!(result.len(), 1)
    }

    #[cfg(all(feature = "agency", feature = "pool_tests", feature = "wallet_backup"))]
    #[test]
    fn test_download_agent_messages() {
        let _setup = SetupConsumer::init();

        // AS CONSUMER GET MESSAGES
        let all_messages = download_agent_messages(None, None).unwrap();
        assert_eq!(all_messages.len(), 0);

        let _wallet_backup = crate::wallet_backup::create_wallet_backup("123", crate::settings::DEFAULT_WALLET_KEY).unwrap();

        thread::sleep(Duration::from_millis(2000));
        let all_messages = download_agent_messages(None, None).unwrap();
        assert_eq!(all_messages.len(), 1);

        let invalid_status_code = "abc".to_string();
        let bad_req = download_agent_messages(Some(vec![invalid_status_code]), None);
        assert!(bad_req.is_err());
    }

    #[cfg(all(feature = "agency", feature = "pool_tests"))]
    #[test]
    fn test_download_messages() {
        let _setup = SetupLibraryAgencyV2NewProvisioning::init();

        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (_faber, alice) = crate::connection::tests::create_connected_connections();

        let (_, cred_def_handle) = crate::credential_def::tests::create_cred_def_real(false);

        let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
        let credential_offer = crate::issuer_credential::issuer_credential_create(cred_def_handle,
                                                                             "1".to_string(),
                                                                             institution_did.clone(),
                                                                             "credential_name".to_string(),
                                                                             credential_data.to_owned(),
                                                                             1).unwrap();

        credential_offer.send_credential_offer(alice).unwrap();

        thread::sleep(Duration::from_millis(1000));

        let hello_uid = alice.send_generic_message("hello", &json!({"msg_type":"hello", "msg_title": "hello", "ref_msg_id": null}).to_string()).unwrap();

        // AS CONSUMER GET MESSAGES
        crate::utils::devsetup::set_consumer();

        thread::sleep(Duration::from_millis(3000));

        let _all_messages = download_messages(None, None, None).unwrap();

        let pending = download_messages(None, Some(vec!["MS-103".to_string()]), None).unwrap();
        assert_eq!(pending.len(), 1);
        assert!(pending[0].msgs[0].decrypted_payload.is_some());

        let accepted = download_messages(None, Some(vec!["MS-104".to_string()]), None).unwrap();
        assert_eq!(accepted[0].msgs.len(), 2);

        let specific = download_messages(None, None, Some(vec![accepted[0].msgs[0].uid.clone()])).unwrap();
        assert_eq!(specific.len(), 1);

        // No pending will return empty list
        let empty = download_messages(None, Some(vec!["MS-103".to_string()]), Some(vec![accepted[0].msgs[0].uid.clone()])).unwrap();
        assert_eq!(empty.len(), 1);

        let hello_msg = download_messages(None, None, Some(vec![hello_uid])).unwrap();
        assert_eq!(hello_msg[0].msgs[0].decrypted_payload, Some("{\"@type\":{\"name\":\"hello\",\"ver\":\"1.0\",\"fmt\":\"json\"},\"@msg\":\"hello\"}".to_string()));

        // Agency returns a bad request response for invalid dids
        let invalid_did = "abc".to_string();
        let bad_req = download_messages(Some(vec![invalid_did]), None, None);
        assert_eq!(bad_req.unwrap_err().kind(), VcxErrorKind::InvalidAgencyRequest);
    }

    #[cfg(all(feature = "agency", feature = "pool_tests"))]
    #[test]
    fn test_download_message() {
        let _setup = SetupLibraryAgencyV2NewProvisioning::init();

        let (_faber, alice) = crate::connection::tests::create_connected_connections();

        let message = "hello";
        let message_options = json!({"msg_type":"hello", "msg_title": "hello", "ref_msg_id": null}).to_string();
        let hello_uid = alice.send_generic_message(message, &message_options).unwrap();

        // AS CONSUMER GET MESSAGE
        crate::utils::devsetup::set_consumer();

        thread::sleep(Duration::from_secs(5));
        // download hello message
        let retrieved_message = download_message(hello_uid).unwrap();

        let expected_payload = json!({"@type":{"name":"hello","ver":"1.0","fmt":"json"},"@msg":"hello"});
        let retrieved_payload: serde_json::Value = serde_json::from_str(&retrieved_message.decrypted_payload.unwrap()).unwrap();
        assert_eq!(retrieved_payload, expected_payload);

        // download unknown message
        let unknown_uid = "unknown";
        let res = download_message(unknown_uid.to_string()).unwrap_err();
        assert_eq!(VcxErrorKind::InvalidAgencyResponse, res.kind())
    }
}
