use crate::settings;
use crate::messages::{A2AMessage, A2AMessageV2, A2AMessageKinds, prepare_message_for_agent_v2};
use crate::messages::message_type::MessageTypes;
use crate::error::VcxResult;
use crate::utils::httpclient;
//use messages::wallet_backup::{prepare_message_for_agency_v2};

#[derive(Serialize, Deserialize, Debug)]
pub struct Backup {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
    wallet: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackupBuilder {
    wallet: String,
}

impl BackupBuilder {
    pub fn create() -> BackupBuilder {
        BackupBuilder {
            wallet: String::new(),
        }
    }

    pub fn wallet_data(&mut self, wallet_data: Vec<u8>) -> &mut Self {
        self.wallet = base64::encode(&wallet_data);
        self
    }

    pub fn send_secure(&mut self) -> VcxResult<()> {
        trace!("BackupBuilder::send >>>");

        let data = self.prepare_request()?;

        httpclient::post_u8(&data)?;

        Ok(())
    }

    fn prepare_request(&self) -> VcxResult<Vec<u8>> {
        trace!("BackupBuilder::prepare_request >>>");

        let message = A2AMessage::Version2( A2AMessageV2::Backup(
            Backup {
                msg_type: MessageTypes::MessageTypeV2(MessageTypes::build_v2(
                    A2AMessageKinds::Backup
                )),
                wallet: self.wallet.clone(),
            }
        ));

        let agency_did = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?;
        let agency_vk = settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY)?;
        let my_vk = settings::get_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY)?;

        trace!("BackupBuilder::prepare_request >>> message: {:?}", secret!(message));

        prepare_message_for_agent_v2(vec![message], &my_vk, &agency_did, &agency_vk)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BackupAck {
    #[serde(rename = "@type")]
    msg_type: MessageTypes,
}

#[cfg(feature = "wallet_backup")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::{wallet_backup_init, backup_wallet, RemoteMessageType};
    use crate::settings::{CONFIG_PROTOCOL_TYPE};
    use crate::utils::libindy::signus::create_and_store_my_did;
    use crate::utils::constants::{MY1_SEED, MY2_SEED, MY3_SEED};
    use std::thread;
    use std::time::Duration;
    use crate::messages::wallet_backup::received_expected_message;
    use crate::utils::devsetup::*;

    #[cfg(feature = "wallet_backup")]
    #[test]
    fn test_wallet_backup() {
        let _setup = SetupLibraryWalletPool::init();

        let (_, agent_vk) = create_and_store_my_did(Some(MY2_SEED), None).unwrap();
        let (_, my_vk) = create_and_store_my_did(Some(MY1_SEED), None).unwrap();
        let (_, agency_vk) = create_and_store_my_did(Some(MY3_SEED), None).unwrap();

        settings::set_config_value(settings::CONFIG_AGENCY_VERKEY, &agency_vk);
        settings::set_config_value(settings::CONFIG_REMOTE_TO_SDK_VERKEY, &agent_vk);
        settings::set_config_value(settings::CONFIG_SDK_TO_REMOTE_VERKEY, &my_vk);

        settings::set_config_value(CONFIG_PROTOCOL_TYPE, &settings::protocol::ProtocolTypes::V2.to_string());

        let msg = backup_wallet()
            .wallet_data(vec![1, 2, 3])
            .prepare_request().unwrap();
        assert!(msg.len() > 0);

    }

    #[cfg(feature = "wallet_backup")]
    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_backup_real() {
        let _setup = SetupConsumer::init();

        wallet_backup_init()
            .recovery_vk(settings::CONFIG_WALLET_BACKUP_KEY).unwrap()
            .dead_drop_address(settings::CONFIG_WALLET_BACKUP_KEY).unwrap()
            .cloud_address(settings::CONFIG_REMOTE_TO_SDK_DID.as_bytes()).unwrap()
            .send_secure().unwrap();
        thread::sleep(Duration::from_millis(2000));

        assert!(backup_wallet().wallet_data(vec![1, 2, 3]).send_secure().is_ok());
    }

    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    #[test]
    fn test_received_backup_ack_true() {
        let _setup = SetupConsumer::init();

        wallet_backup_init()
            .recovery_vk(settings::CONFIG_WALLET_BACKUP_KEY).unwrap()
            .dead_drop_address(settings::CONFIG_WALLET_BACKUP_KEY).unwrap()
            .cloud_address(settings::CONFIG_REMOTE_TO_SDK_DID.as_bytes()).unwrap()
            .send_secure().unwrap();
        thread::sleep(Duration::from_millis(2000));

        assert!(backup_wallet().wallet_data(vec![1, 2, 3]).send_secure().is_ok());
        thread::sleep(Duration::from_millis(2000));

        assert_eq!(received_expected_message(None, RemoteMessageType::WalletBackupAck).unwrap(), true);
    }
}
