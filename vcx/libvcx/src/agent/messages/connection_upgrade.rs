use std::collections::HashMap;

use crate::settings;
use crate::utils::httpclient;
use crate::error::prelude::*;
use crate::settings::protocol::ProtocolTypes;
use crate::agent::messages::{A2AMessage, A2AMessageV1, A2AMessageKinds, A2AMessageV2, parse_response_from_agency, prepare_message_for_agency};
use crate::agent::messages::message_type::MessageTypes;

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GetUpgradeInfo {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    #[serde(rename = "pairwiseDIDs")]
    pairwise_dids: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpgradeInfoResponse {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    data: UpgradeInfo,
}

pub type UpgradeInfo = HashMap<String, UpgradeInfoItem>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeInfoItem {
    pub their_agency_endpoint: String,
    pub their_agency_verkey: String,
    pub their_agency_did: String,
}

#[derive(Debug)]
pub struct GetUpgradeInfoBuilder {
    version: ProtocolTypes,
    pairwise_dids: Vec<String>
}

impl GetUpgradeInfoBuilder {
    pub fn create() -> GetUpgradeInfoBuilder {
        trace!("GetUpgradeInfoBuilder::create_message >>>");

        GetUpgradeInfoBuilder {
            version: ProtocolTypes::V3,
            pairwise_dids: Vec::new()
        }
    }

    pub fn for_did(&mut self, did: &str) -> VcxResult<&mut Self> {
        self.pairwise_dids.push(did.to_string());
        Ok(self)
    }

    pub fn send_secure(&self) -> VcxResult<UpgradeInfo> {
        trace!("GetUpgradeInfoBuilder::send_secure >>>");

        let data = self.prepare_request()?;

        let response = httpclient::post_u8(&data)?;

        self.parse_response(&response)
    }

    fn prepare_request(&self) -> VcxResult<Vec<u8>> {
        trace!("GetUpgradeInfoBuilder::prepare_request >>>");

        let message = match self.version {
            ProtocolTypes::V1 =>
                A2AMessage::Version1(
                    A2AMessageV1::GetUpgradeInfo(GetUpgradeInfo {
                        msg_type: MessageTypes::MessageTypeV1(MessageTypes::build_v1(A2AMessageKinds::GetUpgradeInfo)),
                        pairwise_dids: self.pairwise_dids.clone(),
                    })
                ),
            ProtocolTypes::V2 |
            ProtocolTypes::V3 |
            ProtocolTypes::V4 =>
                A2AMessage::Version2(
                    A2AMessageV2::GetUpgradeInfo(GetUpgradeInfo {
                        msg_type: MessageTypes::MessageTypeV2(MessageTypes::build_v2(A2AMessageKinds::GetUpgradeInfo)),
                        pairwise_dids: self.pairwise_dids.clone(),
                    })
                ),
        };

        trace!("GetConnectionUpgradeInfoBuilder::prepare_request >>> message: {:?}", secret!(message));

        let agency_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;

        prepare_message_for_agency(&message, &agency_did, &self.version)
    }

    fn parse_response(&self, response: &[u8]) -> VcxResult<UpgradeInfo> {
        trace!("GetConnectionUpgradeInfoBuilder::parse_response >>>");

        let mut response = parse_response_from_agency(response, &self.version)?;

        match response.swap_remove(0) {
            A2AMessage::Version1(A2AMessageV1::UpgradeInfoResponse(res)) => Ok(res.data),
            A2AMessage::Version2(A2AMessageV2::UpgradeInfoResponse(res)) => Ok(res.data),
            r => Err(VcxError::from_msg(VcxErrorKind::InvalidAgencyResponse, format!("Agency response does not match any variant of UpgradeInfo, got: {:#?}", r)))
        }
    }
}

