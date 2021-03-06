use crate::error::{VcxResult, VcxErrorKind, VcxError};
use crate::utils::{
    option_util::get_or_err,
    object_cache::Handle,
};
use crate::settings::{get_config_value, CONFIG_REMOTE_TO_SDK_DID, CONFIG_REMOTE_TO_SDK_VERKEY, CONFIG_AGENCY_DID, CONFIG_AGENCY_VERKEY};

use crate::{connection::Connections};
use crate::settings::protocol::ProtocolTypes;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MyAgentInfo {
    pub connection_handle: Option<Handle<Connections>>,
    pub my_pw_did: Option<String>,
    pub my_pw_vk: Option<String>,
    pub their_pw_did: Option<String>,
    pub their_pw_vk: Option<String>,
    pub pw_agent_did: Option<String>,
    pub pw_agent_vk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<ProtocolTypes>,

    // User Agent
    pub agent_did: String,
    pub agent_vk: String,
    pub agency_did: String,
    pub agency_vk: String,
}

pub fn get_agent_attr(v: &Option<String>) -> VcxResult<String> { get_or_err(v, VcxErrorKind::NoAgentInformation) }

impl MyAgentInfo {
    pub fn connection_handle(&self) -> VcxResult<Handle<Connections>> {
        self.connection_handle
            .ok_or(VcxError::from(VcxErrorKind::InvalidConnectionHandle))
    }

    fn retrieve(&self,
                value: &Option<String>,
                getter: fn(Handle<Connections>) -> VcxResult<String>) -> VcxResult<String> {
        value
            .as_ref()
            .map(|x| Ok(x.to_string()))
            .unwrap_or(getter(self.connection_handle()?))
    }

    pub fn my_pw_did(&self) -> VcxResult<String> { self.retrieve(&self.my_pw_did, Handle::get_pw_did) }

    pub fn my_pw_vk(&self) -> VcxResult<String> { self.retrieve(&self.my_pw_vk, Handle::get_pw_verkey) }

    pub fn their_pw_did(&self) -> VcxResult<String> { self.retrieve(&self.their_pw_did, Handle::get_their_pw_did) }

    pub fn their_pw_vk(&self) -> VcxResult<String> { self.retrieve(&self.their_pw_vk, Handle::get_their_pw_verkey) }

    pub fn pw_agent_did(&self) -> VcxResult<String> { self.retrieve(&self.pw_agent_did, Handle::get_agent_did) }

    pub fn pw_agent_vk(&self) -> VcxResult<String> { self.retrieve(&self.pw_agent_vk, Handle::get_agent_verkey) }

    pub fn version(&self) -> VcxResult<Option<ProtocolTypes>> { self.connection_handle()?.get_version() }

    pub fn pw_info(&mut self, handle: Handle<Connections>) -> VcxResult<MyAgentInfo> {
        self.my_pw_did = Some(handle.get_pw_did()?);
        self.my_pw_vk = Some(handle.get_pw_verkey()?);
        self.their_pw_did = Some(handle.get_their_pw_did()?);
        self.their_pw_vk = Some(handle.get_their_pw_verkey()?);
        self.pw_agent_did = Some(handle.get_agent_did()?);
        self.pw_agent_vk = Some(handle.get_agent_verkey()?);
        self.version = handle.get_version()?;
        self.connection_handle = Some(handle);
        self.log();

        Ok(self.clone())
    }

    fn log(&self) {
        debug!("my_pw_did: {:?} -- my_pw_vk: {:?} -- their_pw_did: {:?} -- pw_agent_did: {:?} \
        -- pw_agent_vk: {:?} -- their_pw_vk: {:?}-- agent_did: {} -- agent_vk: {} -- version: {:?}",
               secret!(self.my_pw_did),
               secret!(self.my_pw_vk),
               secret!(self.their_pw_did),
               secret!(self.their_pw_vk),
               secret!(self.pw_agent_did),
               secret!(self.pw_agent_vk),
               secret!(self.agent_did),
               secret!(self.agent_vk),
               self.version,
        );
    }
}

pub fn get_agent_info() -> VcxResult<MyAgentInfo> {
    Ok(MyAgentInfo {
        connection_handle: None,
        my_pw_did: None,
        my_pw_vk: None,
        their_pw_did: None,
        their_pw_vk: None,
        pw_agent_did: None,
        pw_agent_vk: None,
        version: None,
        agent_did: get_config_value(CONFIG_REMOTE_TO_SDK_DID)?,
        agent_vk: get_config_value(CONFIG_REMOTE_TO_SDK_VERKEY)?,
        agency_did: get_config_value(CONFIG_AGENCY_DID)?,
        agency_vk: get_config_value(CONFIG_AGENCY_VERKEY)?,
    })
}

