use api::WalletBackupState;
use settings;
use messages;
use object_cache::ObjectCache;
use error::prelude::*;
use utils::error;
use utils::libindy::wallet::{export, get_wallet_handle, RestoreWalletConfigs, add_record, get_record, WalletRecord};
use utils::libindy::crypto::{create_key, sign, pack_message};
use utils::constants::{DEFAULT_SERIALIZE_VERSION};
use std::path::Path;
use std::fs;
use messages::{RemoteMessageType, retrieve_dead_drop, parse_message_from_response, wallet_backup_restore};
use messages::wallet_backup::received_expected_message;
use messages::get_message::Message;
use utils::openssl::sha256_hex;
use std::io::{Write, Error};
use utils::libindy::wallet;
use std::path::PathBuf;
use settings::test_agency_mode_enabled;

lazy_static! {
    static ref WALLET_BACKUP_MAP: ObjectCache<WalletBackup> = Default::default();
}

pub static RECOVERY_KEY_TYPE: &str = r#"RECOVERY_KEY"#;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloudAddress {
    version: Option<String>,
    agent_did: String,
    agent_vk: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeadDropAddress {
    pub address: String,
    pub locator: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletBackupKeys {
    pub wallet_encryption_key: String,
    pub recovery_vk: String,
    pub dead_drop_address: DeadDropAddress,
    pub cloud_address: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WalletBackup {
    source_id: String,
    state: WalletBackupState,
    to_did: String, // user agent did
    uuid: Option<String>,
    pub keys: WalletBackupKeys,
    has_stored_backup: bool,
}

impl CloudAddress {
    fn to_string(&self) -> VcxResult<String> {
        messages::ObjectWithVersion::new(DEFAULT_SERIALIZE_VERSION, self.to_owned())
            .serialize()
            .map_err(|err| err.extend("Cannot serialize CloudAddress"))
    }

    fn from_str(data: &str) -> VcxResult<CloudAddress> {
        messages::ObjectWithVersion::deserialize(data)
            .map(|obj: messages::ObjectWithVersion<CloudAddress>| obj.data)
            .map_err(|err| err.extend("Cannot deserialize CloudAddress"))
    }
}

impl WalletBackup {

    fn get_source_id(&self) -> &String { &self.source_id }

    fn has_stored_backup(&self) -> bool {
        trace!("WalletBackup::has_cloud_backup >>>");
        self.has_stored_backup
    }

    fn set_state(&mut self, state: WalletBackupState) {
        trace!("WalletBackup::set_state: {:?} >>>", state);
        self.state = state
    }

    fn get_state(&self) -> u32 {
        trace!("WalletBackup::get_state >>>");
        self.state as u32
    }

    fn update_state(&mut self, message: Option<Message>) -> VcxResult<u32> {
        debug!("updating state for wallet_backup {}", self.source_id);
        if test_agency_mode_enabled() { return Ok(self.get_state()) }

        match self.state {
            WalletBackupState::InitRequested =>
                if received_expected_message(message, RemoteMessageType::WalletBackupProvisioned)? {
                    self.state = WalletBackupState::ReadyToExportWallet
                },
            WalletBackupState::BackupInProgress =>
                if received_expected_message(message, RemoteMessageType::WalletBackupAck)? {
                    self.has_stored_backup = true;
                    self.state = WalletBackupState::ReadyToExportWallet
                },
            _ => ()
        }
        Ok(self.get_state())
    }

    pub fn create(source_id: &str, wallet_encryption_key: &str) -> VcxResult<WalletBackup> {
        Ok(WalletBackup {
            source_id: source_id.to_string(),
            state: WalletBackupState::Uninitialized,
            to_did: settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?,
            keys: gen_keys(wallet_encryption_key)?,
            uuid: None,
            has_stored_backup: false
        })
    }

    fn init_backup(&mut self) -> VcxResult<u32> {
        trace!("init_backup >>> ");

        messages::wallet_backup_init()
            .recovery_vk(&self.keys.recovery_vk)?
            .dead_drop_address(&self.keys.dead_drop_address.address)?
            .cloud_address(&self.keys.cloud_address)?
            .send_secure()?;

        self.state = WalletBackupState::InitRequested;

       Ok(error::SUCCESS.code_num)
    }

    fn backup(&mut self, exported_wallet_path: &str) -> VcxResult<u32> {
        let wallet_data = WalletBackup::_retrieve_exported_wallet(&self.keys.wallet_encryption_key, exported_wallet_path)?;

        messages::backup_wallet()
            .wallet_data(wallet_data)
            .send_secure()?;

        self.state = WalletBackupState::BackupInProgress;

        Ok(error::SUCCESS.code_num)
    }

    fn _retrieve_exported_wallet(backup_key: &str, exported_wallet_path: &str) -> VcxResult<Vec<u8>> {
        if settings::test_indy_mode_enabled() { return Ok(Vec::new()) }

        let path = Path::new(exported_wallet_path);
        fs::remove_file(path).unwrap_or(());

        export(get_wallet_handle(), &path, backup_key)?;
        let data = fs::read(&path).map_err(|err| VcxError::from(VcxErrorKind::RetrieveExportedWallet))?;
        fs::remove_file(path).map_err(|err| VcxError::from(VcxErrorKind::RetrieveExportedWallet))?;

        Ok(data)
    }

    fn to_string(&self) -> VcxResult<String> {
        trace!("WalletBackup::to_string >>>");
        messages::ObjectWithVersion::new(DEFAULT_SERIALIZE_VERSION, self.to_owned())
            .serialize()
            .map_err(|err| err.extend("Cannot serialize WalletBackup"))
    }

    fn from_str(data: &str) -> VcxResult<WalletBackup> {
        trace!("WalletBackup::from_str >>> data: {}", secret!(&data));
        messages::ObjectWithVersion::deserialize(data)
            .map(|obj: messages::ObjectWithVersion<WalletBackup>| obj.data)
            .map_err(|err| err.extend("Cannot deserialize WalletBackup"))
    }
}

pub fn create_wallet_backup(source_id: &str, wallet_encryption_key: &str) -> VcxResult<u32> {
    info!("create_wallet_backup >>> source_id: {}", source_id);

    let mut wb = WalletBackup::create(source_id, wallet_encryption_key)?;

    wb.init_backup()?;

    WALLET_BACKUP_MAP.add(wb)
        .or(Err(VcxError::from(VcxErrorKind::CreateWalletBackup)))
}

fn gen_keys(wallet_encryption_key: &str) -> VcxResult<WalletBackupKeys> {
    info!("gen_keys >>> encryption_key: ***");

    let vk = &gen_vk(wallet_encryption_key)?;

    println!("gen_keys: vk: {:?}", vk);
    Ok(WalletBackupKeys {
        wallet_encryption_key: wallet_encryption_key.to_string(),
        recovery_vk: vk.to_string(),
        dead_drop_address: gen_deaddrop_address(vk)?,
        cloud_address: gen_cloud_address(vk)?,
    })
}

fn gen_vk(wallet_encryption_key: &str) -> VcxResult<String> {
    if settings::test_indy_mode_enabled() { return Ok(settings::DEFAULT_WALLET_BACKUP_KEY.to_string()) }

    let vk_seed = sha256_hex(wallet_encryption_key.as_bytes());

    create_key(Some(&vk_seed), None)
        .and_then(|v| _add_generated_vk(&wallet_encryption_key, &v))
        .or_else(|e| _handle_duplicate_vk(e, &wallet_encryption_key) )
}

fn _add_generated_vk(id: &str, vk: &str) -> VcxResult<String> {
    add_record(RECOVERY_KEY_TYPE, id, vk, None)
        .and_then(|()| Ok(vk.to_string()))
}

fn _handle_duplicate_vk(err: VcxError, id: &str) -> VcxResult<String> {
    if &err.kind() == &VcxErrorKind::DuplicationWalletRecord {
        let options = json!({"retrieveType": false, "retrieveValue": true, "retrieveTags": false});
        let record = get_record(RECOVERY_KEY_TYPE, id, &options.to_string())?;
        Ok(WalletRecord::from_str(&record)?.value.unwrap_or(String::new()))
    } else { Err(err) }
}

fn gen_deaddrop_address(vk: &str) -> VcxResult<DeadDropAddress> {
    info!("gen_deaddrop_address >>> vk: {}", vk);
    if settings::test_indy_mode_enabled() { return Ok(DeadDropAddress {address: String::new(), locator: String::new()}) }

    let locator = sha256_hex(&sign(vk, "wallet-backup".as_bytes())?);
    Ok(DeadDropAddress {
        locator: locator.to_string(),
        address: sha256_hex((vk.to_string() + &locator).as_bytes()),
    })

}

fn gen_cloud_address(vk: &str) -> VcxResult<Vec<u8>> {
    info!("gen_cloud_address >>> vk: {}", vk);
    if settings::test_indy_mode_enabled() { return Ok(Vec::new()) }
    let cloud_address = CloudAddress {
        version: None,
        agent_did: settings::get_config_value(::settings::CONFIG_REMOTE_TO_SDK_DID)?,
        agent_vk: settings::get_config_value(::settings::CONFIG_REMOTE_TO_SDK_VERKEY)?
    };

    let receiver_keys = json!([vk]).to_string();
    pack_message(None, &receiver_keys, cloud_address.to_string()?.as_bytes())
}

/*
    Todo: exported_wallet_path is needed because the only exposed libindy functionality for exporting
    an encrypted wallet, writes it to the file system. A possible better way is for libindy's export_wallet
    to optionally return an encrypted stream of bytes instead of writing it to the fs. This could also
    be done in a separate libindy api call if necessary.
 */
pub fn backup_wallet(handle: u32, exported_wallet_path: &str) -> VcxResult<u32> {
    info!("backup_wallet >>> handle: {}, export_path: {}", handle, exported_wallet_path);
    WALLET_BACKUP_MAP.get_mut(handle, |wb| {
        wb.backup(exported_wallet_path)
    })
}

pub fn restore_wallet(config: &str) -> VcxResult<()> {
    info!("restore_wallet >>> config: ***");
    let (restore_config, backup) = restore_from_cloud(config)?;

    reconstitute_restored_wallet(config, &restore_config, &backup)?;

    Ok(())
}

fn restore_from_cloud(config: &str) -> VcxResult<(RestoreWalletConfigs, Vec<u8>)> {
    let recovery_config = RestoreWalletConfigs::from_str(config)?;
    let recovery_vk  = gen_vk(&recovery_config.backup_key)?;
    let cloud_address = recover_dead_drop(&recovery_vk)?;
    let backup = wallet_backup_restore()
        .recovery_vk(&recovery_vk)?
        .agent_did(&cloud_address.agent_did)?
        .agent_vk(&cloud_address.agent_vk)?
        .send_secure()?;

    let encrypted_wallet = base64::decode(&backup.wallet)
        .map_err(|e| VcxError::from_msg(VcxErrorKind::RetrieveExportedWallet, format!("Encrypted wallet not base64 encoded: {:?}", e)))?;

    Ok((recovery_config, encrypted_wallet))
}

fn reconstitute_restored_wallet(config: &str, recovery_config: &RestoreWalletConfigs, encrypted_wallet: &[u8]) -> VcxResult<()> {
    _write_encrypted_wallet_for_import(&recovery_config.exported_wallet_path, encrypted_wallet)?;

    info!("Deleting temporary wallet before the recovered wallet is imported");
    wallet::delete_wallet(&settings::get_config_value(settings::CONFIG_WALLET_NAME)?, None, None, None)?;

    wallet::import(config)?;

    //Todo: Fix libindy
    // Deletes recovered encrypted wallet from the temporary location on the file system
    // This will be removed once libindy enables import/export without file system location
    let path = Path::new(&recovery_config.exported_wallet_path);
    fs::remove_file(path).map_err(|err| VcxError::from(VcxErrorKind::RetrieveExportedWallet))?;

    wallet::open_wallet(&recovery_config.wallet_name, None, None, None)?;
    Ok(())
}

fn _write_encrypted_wallet_for_import(path: &str, wallet: &[u8]) -> VcxResult<()> {
    let err = |e: Error| VcxError::from_msg( VcxErrorKind::IOError, format!("Wallet IO error: {:?}", e));

    let path = PathBuf::from(path);

    if let Some(parent_path) = path.parent() {
        fs::DirBuilder::new()
            .recursive(true)
            .create(parent_path).map_err(err)?;
    }

    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .truncate(true)
        .open(path).map_err(err)?
        .write_all(wallet).map_err(err)?;

    Ok(())
}

pub fn recover_dead_drop(vk: &str) -> VcxResult<CloudAddress> {
    info!("recover_dead_drop >>> vk: ***");
    let dead_drop_info = gen_deaddrop_address(&vk)?;
    let locator_sig = sign(&vk, dead_drop_info.locator.as_bytes())?;

    let dead_drop_result = retrieve_dead_drop()
        .recovery_vk(&vk).unwrap()
        .dead_drop_address(&dead_drop_info.address).unwrap()
        .locator(&dead_drop_info.locator).unwrap()
        .signature(&locator_sig).unwrap()
        .send_secure()?;

    let entry = dead_drop_result.entry.ok_or(VcxErrorKind::RetrieveDeadDrop)?;
    let encrypted_ca = base64::decode(&entry.data)
        .map_err(|_| VcxError::from_msg(VcxErrorKind::RetrieveDeadDrop, "Cloud Address not base64 encoded"))?;

    CloudAddress::from_str(&parse_message_from_response(&encrypted_ca)?)
}

pub fn is_valid_handle(handle: u32) -> bool { WALLET_BACKUP_MAP.has_handle(handle) }

pub fn get_state(handle: u32) -> u32 {
    WALLET_BACKUP_MAP.get(handle, |wb| {
        debug!("get state for wallet_backup {}", wb.get_source_id());
        Ok(wb.get_state().clone())
    }).unwrap_or(WalletBackupState::Uninitialized as u32)
}

pub fn get_source_id(handle: u32) -> VcxResult<String> {
    WALLET_BACKUP_MAP.get(handle, |wb| {
        Ok(wb.get_source_id().clone())
    }).or(Err(VcxError::from(VcxErrorKind::InvalidHandle)))
}

pub fn to_string(handle: u32) -> VcxResult<String> {
    WALLET_BACKUP_MAP.get(handle, |obj| {
        WalletBackup::to_string(&obj)
    })
}

pub fn from_string(wallet_backup_data: &str) -> VcxResult<u32> {
    let wallet_backup: WalletBackup = WalletBackup::from_str(wallet_backup_data)?;

    let new_handle = WALLET_BACKUP_MAP.add(wallet_backup)?;

    info!("inserting handle {} into wallet backup table", new_handle);

    Ok(new_handle)
}

pub fn set_state(handle: u32, state: WalletBackupState) -> VcxResult<()> {
    WALLET_BACKUP_MAP.get_mut(handle, |wb| {
        Ok(wb.set_state(state))
    })
}

pub fn update_state(handle: u32, message: Option<Message>) -> VcxResult<u32> {
    info!("update_state >>> source_id {}", get_source_id(handle)?);
    WALLET_BACKUP_MAP.get_mut(handle, |wb| {
        wb.update_state(message.clone())
    })
}

pub fn has_known_cloud_backup(handle: u32) -> bool {
    WALLET_BACKUP_MAP.get(handle, |wb| {
        Ok(wb.has_stored_backup().clone())
    }).unwrap_or(false)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use utils::devsetup::tests::setup_wallet_env;
    use serde_json::Value;
    use std::thread;
    use std::time::Duration;
    use utils::libindy::wallet;
    use std::fs::File;
    use utils::devsetup::tests::{test_wallet, set_consumer};
    use utils::devsetup::tests::{cleanup_local_env, setup_local_env};

    pub const WALLET_PROVISION_AGENT_RESPONSE: &'static [u8; 2] = &[79, 75];
    static SOURCE_ID: &str = r#"12345"#;
    static FILE_PATH: &str = r#"/tmp/tmp_wallet"#;
    pub static BACKUP_KEY: &str = r#"8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY"#;
    pub static RECORD_TYPE: &str = r#"cloudBackupType"#;
    pub static ID: &str = r#"cloudBackupId"#;
    pub static RECORD_VALUE: &str = r#"save before cloud backup"#;
    pub static PATH: &str = r#"/tmp/cloud_backup.zip"#;


    pub struct TestBackupData {
        pub wb_handle: u32,
        pub recovery_vk: String,
        pub dd_address: String,
        pub locator: String,
        pub encryption_key: String,
        pub cloud_address: Vec<u8>,
        pub sig: Vec<u8>,
    }

    impl TestBackupData {
        pub fn new(handle: Option<u32>, vk: Option<String>, dd_address: Option<String>,
                   locator: Option<String>, cloud_address: Option<Vec<u8>>, sig: Option<Vec<u8>>, key: Option<String>) -> TestBackupData {
            TestBackupData {
                wb_handle: handle.unwrap_or_default(),
                recovery_vk: vk.unwrap_or_default(),
                dd_address: dd_address.unwrap_or_default(),
                locator: locator.unwrap_or_default(),
                cloud_address: cloud_address.unwrap_or_default(),
                sig: sig.unwrap_or_default(),
                encryption_key: key.unwrap_or(BACKUP_KEY.to_string()),
            }
        }
    }

    pub fn restore_config(path: Option<String>) -> RestoreWalletConfigs {
        RestoreWalletConfigs {
            wallet_name: test_wallet(),
            wallet_key: BACKUP_KEY.to_string(),
            exported_wallet_path: path.unwrap_or(PATH.to_string()),
            backup_key: BACKUP_KEY.to_string(),
            key_derivation: None,
        }
    }

    pub fn init_backup() -> TestBackupData {
        let mut wb = WalletBackup::create(SOURCE_ID, BACKUP_KEY).unwrap();
        wb.init_backup().unwrap();

        let k = wb.keys.clone();
        let dd = k.dead_drop_address.clone();
        let sig = sign(&k.recovery_vk, dd.locator.as_bytes()).unwrap();

        let wb_handle = WALLET_BACKUP_MAP.add(wb).unwrap();

        TestBackupData::new(Some(wb_handle),
                            Some(k.recovery_vk.to_string()),
                            Some(dd.address.clone()),
                            Some(dd.locator.clone()),
                            Some(k.cloud_address.clone()),
                            Some(sig),
                                Some(BACKUP_KEY.to_string()),
                            )

    }

    pub fn backup_wallet_utils() -> TestBackupData {
        wallet::add_record(RECORD_TYPE, ID, RECORD_VALUE, None).unwrap();
        let wb = init_backup();

        backup_wallet(wb.wb_handle, PATH).unwrap();
        thread::sleep(Duration::from_millis(1000));

        wb

    }

    mod create_wallet_backup {
       use super::*;

        #[cfg(feature = "wallet_backup")]
        #[cfg(feature = "agency")]
        #[cfg(feature = "pool_tests")]
        #[test]
        fn create_backup_succeeds_real() {
            init!("agency");
            set_consumer();

            assert!(create_wallet_backup(SOURCE_ID, BACKUP_KEY).is_ok());

            teardown!("agency");
        }

        #[cfg(feature = "wallet_backup")]
        #[cfg(feature = "agency")]
        #[cfg(feature = "pool_tests")]
        #[test]
        fn create_two_backup_init_succeeds_real() {
            init!("agency");
            set_consumer();

            assert!(create_wallet_backup(SOURCE_ID, BACKUP_KEY).is_ok());
            assert!(create_wallet_backup(SOURCE_ID, BACKUP_KEY).is_ok());

            teardown!("agency");
        }
    }

    mod update_state {
        use super::*;

        #[cfg(feature = "wallet_backup")]
        #[test]
        fn update_state_success() {
            init!("true");
            ::utils::httpclient::set_next_u8_response(WALLET_PROVISION_AGENT_RESPONSE.to_vec());

            let handle = create_wallet_backup(SOURCE_ID, BACKUP_KEY).unwrap();
            assert!(update_state(handle, None).is_ok());
            assert_eq!(get_state(handle), WalletBackupState::InitRequested as u32);
        }

        #[cfg(feature = "wallet_backup")]
        #[cfg(feature = "agency")]
        #[cfg(feature = "pool_tests")]
        #[test]
        fn update_state_with_provisioned_msg_changes_state_to_ready_to_export() {
            init!("agency");
            set_consumer();

            let handle = create_wallet_backup(SOURCE_ID, BACKUP_KEY).unwrap();
            thread::sleep(Duration::from_millis(2000));

            assert!(update_state(handle, None).is_ok());
            assert_eq!(get_state(handle), WalletBackupState::ReadyToExportWallet as u32);
            teardown!("agency");
        }

        #[cfg(feature = "wallet_backup")]
        #[cfg(feature = "agency")]
        #[cfg(feature = "pool_tests")]
        #[test]
        fn update_state_with_backup_ack_msg_changes_state_to_ready_to_export() {
            init!("agency");

            set_consumer();
            let handle = create_wallet_backup(SOURCE_ID, BACKUP_KEY).unwrap();
            thread::sleep(Duration::from_millis(2000));

            assert!(update_state(handle, None).is_ok());
            assert_eq!(get_state(handle), WalletBackupState::ReadyToExportWallet as u32);

            backup_wallet(handle, FILE_PATH).unwrap();
            assert_eq!(get_state(handle), WalletBackupState::BackupInProgress as u32);

            assert!(update_state(handle, None).is_ok());
            assert_eq!(get_state(handle), WalletBackupState::ReadyToExportWallet as u32);
            teardown!("agency");
        }
    }

    mod serialization {
        use super::*;

        #[cfg(feature = "wallet_backup")]
        #[test]
        fn to_string_test() {
            init!("true");
            ::utils::httpclient::set_next_u8_response(WALLET_PROVISION_AGENT_RESPONSE.to_vec());

            let handle = create_wallet_backup(SOURCE_ID, BACKUP_KEY).unwrap();
            let serialized = to_string(handle).unwrap();
            let j: Value = serde_json::from_str(&serialized).unwrap();
            assert_eq!(j["version"], "1.0");
            WalletBackup::from_str(&serialized).unwrap();
        }

        #[cfg(feature = "wallet_backup")]
        #[test]
        fn test_deserialize_fails() {
            assert_eq!(from_string("{}").unwrap_err().kind(), VcxErrorKind::InvalidJson);
        }
    }

    mod backup_wallet {
        use super::*;

        #[cfg(feature = "wallet_backup")]
        #[test]
        fn retrieving_exported_wallet_data_successful() {
            init!("true");
            setup_wallet_env(settings::DEFAULT_WALLET_NAME).unwrap();

            let data = WalletBackup::_retrieve_exported_wallet(BACKUP_KEY, FILE_PATH);

            assert!(data.unwrap().len() > 0);
        }

        #[cfg(feature = "wallet_backup")]
        #[test]
        fn retrieve_exported_wallet_success_with_file_already_created() {
            init!("true");
            File::create(FILE_PATH).and_then(|mut f| f.write_all(&vec![1, 2, 3])).unwrap();

            setup_wallet_env(settings::DEFAULT_WALLET_NAME).unwrap();

            assert!(WalletBackup::_retrieve_exported_wallet(BACKUP_KEY, FILE_PATH).is_ok());
        }

        #[cfg(feature = "wallet_backup")]
        #[test]
        fn backup_wallet_fails_with_invalid_handle() {
            init!("true");
            assert_eq!(backup_wallet(0, FILE_PATH).unwrap_err().kind(), VcxErrorKind::InvalidHandle)
        }

        #[cfg(feature = "wallet_backup")]
        #[cfg(feature = "agency")]
        #[cfg(feature = "pool_tests")]
        #[test]
        fn backup_wallet_succeeds_real() {
            init!("agency");
            set_consumer();

            let wallet_backup = create_wallet_backup(SOURCE_ID, BACKUP_KEY).unwrap();
            thread::sleep(Duration::from_millis(2000));

            assert_eq!(get_state(wallet_backup), WalletBackupState::InitRequested as u32);
            assert!(update_state(wallet_backup, None).is_ok());

            backup_wallet(wallet_backup, FILE_PATH).unwrap();
            assert_eq!(get_state(wallet_backup), WalletBackupState::BackupInProgress as u32);

            assert!(update_state(wallet_backup, None).is_ok());
            assert_eq!(get_state(wallet_backup), WalletBackupState::ReadyToExportWallet as u32);
            assert!(has_known_cloud_backup(wallet_backup));
            teardown!("agency");
        }

        #[cfg(feature = "wallet_backup")]
        #[cfg(feature = "agency")]
        #[cfg(feature = "pool_tests")]
        #[test]
        fn backup_wallet_multiple_times_real() {
            init!("agency");
            set_consumer();

            let wallet_backup = create_wallet_backup(SOURCE_ID, BACKUP_KEY).unwrap();
            thread::sleep(Duration::from_millis(2000));

            assert_eq!(get_state(wallet_backup), WalletBackupState::InitRequested as u32);
            assert!(update_state(wallet_backup, None).is_ok());

            backup_wallet(wallet_backup, FILE_PATH).unwrap();
            assert_eq!(get_state(wallet_backup), WalletBackupState::BackupInProgress as u32);

            assert!(update_state(wallet_backup, None).is_ok());
            assert_eq!(get_state(wallet_backup), WalletBackupState::ReadyToExportWallet as u32);
            assert!(has_known_cloud_backup(wallet_backup));

            backup_wallet(wallet_backup, FILE_PATH).unwrap();
            assert_eq!(get_state(wallet_backup), WalletBackupState::BackupInProgress as u32);
            teardown!("agency");
        }
    }

    mod restore_wallet {
        use super::*;


        #[cfg(feature = "wallet_backup")]
        #[cfg(feature = "agency")]
        #[cfg(feature = "pool_tests")]
        #[test]
        fn restore_wallet_real() {
            init!("agency");

            set_consumer();
            let wb = backup_wallet_utils();
            cleanup_local_env();

            setup_local_env("1.0");
            set_consumer();

            restore_wallet(&restore_config(None).to_string().unwrap()).unwrap();

            let options = json!({
                "retrieveType": true,
                "retrieveValue": true,
                "retrieveTags": true
            }).to_string();
            let record = wallet::get_record(RECORD_TYPE, ID, &options).unwrap();
            let record: serde_json::Value = serde_json::from_str(&record).unwrap();
            assert_eq!(&record, &json!({"value":RECORD_VALUE, "type": RECORD_TYPE, "id": ID, "tags": {}}));

            wallet::delete_wallet(&restore_config(None).wallet_name, None, None, None).unwrap();
            teardown!("agency");
        }

        #[cfg(feature = "wallet_backup")]
        #[cfg(feature = "agency")]
        #[cfg(feature = "pool_tests")]
        #[test]
        fn restore_wallet_fails_with_no_backup() {
            init!("agency");
            set_consumer();

            wallet::add_record(RECORD_TYPE, ID, RECORD_VALUE, None).unwrap();
            let wb = init_backup();
            thread::sleep(Duration::from_millis(2000));
            cleanup_local_env();

            setup_local_env("1.0");
            set_consumer();

            let rc = restore_wallet(&restore_config(None).to_string().unwrap());
            assert_eq!(
                rc.unwrap_err().to_string(),
                "Error: Message failed in post\n  Caused by: POST failed with: {\"statusCode\":\"GNR-111\",\"statusMsg\":\"No Wallet Backup available to download\"}\n"
            );
            teardown!("agency");
        }

        #[cfg(feature = "wallet_backup")]
        #[cfg(feature = "agency")]
        #[cfg(feature = "pool_tests")]
        #[test]
        fn recovery_creates_file_structure_for_undefined_path_recovery_success() {
            init!("agency");

            set_consumer();
            let wb = backup_wallet_utils();
            cleanup_local_env();

            init!("agency");
            set_consumer();

            // Just to make sure path doesn't exist
            let base = "/tmp/uncreated/";
            let uncreated_path = format!("{}/nested/test.txt", base);
            ::std::fs::remove_dir_all(base).unwrap_or(println!("No Directory to delete before test"));
            let rc = restore_wallet(&restore_config(Some(uncreated_path.to_string())).to_string().unwrap());
            ::std::fs::remove_dir_all(base).unwrap_or(println!("No Directory to delete after test"));

            assert!(rc.is_ok());
            teardown!("agency");
        }

        #[cfg(feature = "wallet_backup")]
        #[cfg(feature = "agency")]
        #[cfg(feature = "pool_tests")]
        #[test]
        fn recovery_overwrites_export_path_when_file_already_exists() {
            init!("agency");
            set_consumer();

            let wb = backup_wallet_utils();
            cleanup_local_env();

            init!("agency");
            set_consumer();

            let base = "/tmp/existing/";
            let existing_file = format!("{}/test.txt", base);
            _write_encrypted_wallet_for_import(&existing_file, &vec![1, 2, 3, 4, 5, 6, 7]).unwrap();
            let rc = restore_wallet(&restore_config(Some(existing_file.to_string())).to_string().unwrap());
            ::std::fs::remove_dir_all(base).unwrap_or(println!("No Directory to delete after test"));

            assert!(rc.is_ok());
            teardown!("agency");
        }
    }
}

