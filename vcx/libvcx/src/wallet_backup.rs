use api::WalletBackupState;
use settings;
use messages;
use object_cache::ObjectCache;
use error::prelude::*;
use utils::error;
use utils::libindy::wallet::{export, get_wallet_handle, RestoreWalletConfigs, add_record, get_record, WalletRecord, import, open_wallet};
use utils::libindy::crypto::{create_key, sign, pack_message};
use utils::constants::DEFAULT_SERIALIZE_VERSION;
use std::path::Path;
use std::fs;
use messages::{RemoteMessageType, retrieve_dead_drop, parse_message_from_response, wallet_backup_restore};
use messages::wallet_backup::received_expected_message;
use messages::get_message::Message;
use utils::openssl::sha256_hex;
use std::io::{Write, Error};
use utils::libindy::wallet;
use std::path::PathBuf;
use rand::Rng;

use crate::object_cache::Handle;

lazy_static! {
    static ref WALLET_BACKUP_MAP: ObjectCache<WalletBackup> = Default::default();
}

pub static RECOVERY_KEY_TYPE: &str = r#"RECOVERY_KEY"#;
pub static MAX_WALLET_SIZE: usize = 5000000;

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
    to_did: String,
    // user agent did
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
        trace!("WalletBackup::update_state >>> message: {:?}", secret!(message));
        debug!("updating state for wallet_backup {}", self.source_id);
        if settings::agency_mocks_enabled() { return Ok(self.get_state()); }

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

        let state = self.get_state();
        trace!("WalletBackup::update_state <<< state: {:?}", state);
        Ok(state)
    }

    pub fn create(source_id: &str, wallet_encryption_key: &str) -> VcxResult<WalletBackup> {
        Ok(WalletBackup {
            source_id: source_id.to_string(),
            state: WalletBackupState::Uninitialized,
            to_did: settings::get_config_value(settings::CONFIG_REMOTE_TO_SDK_DID)?,
            keys: gen_keys(wallet_encryption_key)?,
            uuid: None,
            has_stored_backup: false,
        })
    }

    fn init_backup(&mut self) -> VcxResult<u32> {
        trace!("WalletBackup::init_backup >>> ");
        debug!("initializing wallet backup");

        messages::wallet_backup_init()
            .recovery_vk(&self.keys.recovery_vk)?
            .dead_drop_address(&self.keys.dead_drop_address.address)?
            .cloud_address(&self.keys.cloud_address)?
            .send_secure()?;

        self.state = WalletBackupState::InitRequested;

        trace!("WalletBackup::init_backup <<< ");

        Ok(error::SUCCESS.code_num)
    }

    fn backup(&mut self, exported_wallet_path: &str) -> VcxResult<u32> {
        trace!("WalletBackup::backup >>> exported_wallet_path: {:?}", secret!(exported_wallet_path));
        debug!("sending wallet backup on agent");

        let wallet_data = _read_exported_wallet(&self.keys.wallet_encryption_key, exported_wallet_path)?;

        if wallet_data.len() > MAX_WALLET_SIZE {
            error!("{}: greater than {}", VcxErrorKind::MaxBackupSize, MAX_WALLET_SIZE);
            return Err(VcxError::from(VcxErrorKind::MaxBackupSize).into());
        }

        messages::backup_wallet()
            .wallet_data(wallet_data)
            .send_secure()?;

        self.state = WalletBackupState::BackupInProgress;

        trace!("WalletBackup::backup <<<");

        Ok(error::SUCCESS.code_num)
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

pub fn create_wallet_backup(source_id: &str, wallet_encryption_key: &str) -> VcxResult<Handle<WalletBackup>> {
    info!("create_wallet_backup >>> source_id: {}", source_id);
    debug!("creating wallet backup {}", source_id);

    let mut wb = WalletBackup::create(source_id, wallet_encryption_key)?;

    wb.init_backup()?;

    let handle = WALLET_BACKUP_MAP.add(wb)
        .map_err(|_| VcxError::from(VcxErrorKind::CreateWalletBackup))?;

    info!("create_wallet_backup <<< handle: {}", handle);
    Ok(handle)
}

fn gen_keys(wallet_encryption_key: &str) -> VcxResult<WalletBackupKeys> {
    info!("gen_keys >>> encryption_key: {}", secret!(wallet_encryption_key));
    debug!("generating backup keys");

    let vk = &gen_vk(wallet_encryption_key)?;

    let keys = WalletBackupKeys {
        wallet_encryption_key: wallet_encryption_key.to_string(),
        recovery_vk: vk.to_string(),
        dead_drop_address: gen_deaddrop_address(vk)?,
        cloud_address: gen_cloud_address(vk)?,
    };

    info!("gen_keys <<< keys: {:?}", secret!(keys));
    Ok(keys)
}

fn gen_vk(wallet_encryption_key: &str) -> VcxResult<String> {
    info!("gen_vk >>> wallet_encryption_key: {}", secret!(wallet_encryption_key));

    if settings::agency_mocks_enabled() { return Ok(settings::DEFAULT_WALLET_BACKUP_KEY.to_string()); }

    let vk_seed = sha256_hex(wallet_encryption_key.as_bytes());

    let key = create_key(Some(&vk_seed))
        .and_then(|v| _add_generated_vk(&wallet_encryption_key, &v))
        .or_else(|e| _handle_duplicate_vk(e, &wallet_encryption_key))?;

    info!("gen_vk <<< key: {}", secret!(key));
    Ok(key)
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
    trace!("gen_deaddrop_address >>> vk: {}", secret!(vk));
    info!("generating dead drop address");
    if settings::agency_mocks_enabled() { return Ok(DeadDropAddress { address: String::new(), locator: String::new() }); }

    let locator = sha256_hex(&sign(vk, "wallet-backup".as_bytes())?);
    let dead_drop_addr = DeadDropAddress {
        locator: locator.to_string(),
        address: sha256_hex((vk.to_string() + &locator).as_bytes()),
    };
    trace!("gen_deaddrop_address <<< dead_drop_addr: {:?}", secret!(dead_drop_addr));
    Ok(dead_drop_addr)
}

fn gen_cloud_address(vk: &str) -> VcxResult<Vec<u8>> {
    trace!("gen_cloud_address >>> vk: {}", secret!(vk));
    info!("generating cloud address");

    if settings::agency_mocks_enabled() { return Ok(Vec::new()); }
    let cloud_address = CloudAddress {
        version: None,
        agent_did: settings::get_config_value(::settings::CONFIG_REMOTE_TO_SDK_DID)?,
        agent_vk: settings::get_config_value(::settings::CONFIG_REMOTE_TO_SDK_VERKEY)?,
    };

    let receiver_keys = json!([vk]).to_string();
    pack_message(None, &receiver_keys, cloud_address.to_string()?.as_bytes())
}

fn _read_exported_wallet(backup_key: &str, exported_wallet_path: &str) -> VcxResult<Vec<u8>> {
    trace!("_read_exported_wallet >>> backup_key: {}, exported_wallet_path: {}", secret!(backup_key), secret!(exported_wallet_path));

    if settings::agency_mocks_enabled() { return Ok(Vec::new()); }

    let tmp_dir = _unique_tmp_dir(exported_wallet_path)?;

    let tmp_dir = tmp_dir.to_str()
        .ok_or(VcxError::from_msg(VcxErrorKind::IOError, "Cannot create path"))?;

    export(get_wallet_handle(), tmp_dir, backup_key)?;

    let data = fs::read(tmp_dir)
        .and_then(|data| {
            fs::remove_file(tmp_dir)?;
            Ok(data)
        })
        .map_err(|_| VcxError::from(VcxErrorKind::RetrieveExportedWallet))?;

    trace!("_read_exported_wallet <<<");

    Ok(data)
}


pub fn restore_wallet(config: &str) -> VcxResult<()> {
    trace!("restore_wallet >>> config: {}", secret!(config));
    info!("restoring wallet");
    let (restore_config, backup) = restore_from_cloud(config)?;

    reconstitute_restored_wallet(&restore_config, &backup)
}

fn restore_from_cloud(config: &str) -> VcxResult<(RestoreWalletConfigs, Vec<u8>)> {
    trace!("restore_from_cloud >>> config: {}", secret!(config));

    let recovery_config = RestoreWalletConfigs::from_str(config)?;
    let recovery_vk = gen_vk(&recovery_config.backup_key)?;
    let cloud_address = recover_dead_drop(&recovery_vk)?;
    let backup = wallet_backup_restore()
        .recovery_vk(&recovery_vk)?
        .agent_did(&cloud_address.agent_did)?
        .agent_vk(&cloud_address.agent_vk)?
        .send_secure()?;

    let encrypted_wallet = base64::decode(&backup.wallet)
        .map_err(|e| VcxError::from_msg(VcxErrorKind::RetrieveExportedWallet, format!("Encrypted wallet not base64 encoded: {:?}", e)))?;

    trace!("restore_from_cloud <<<");

    Ok((recovery_config, encrypted_wallet))
}

fn reconstitute_restored_wallet(recovery_config: &RestoreWalletConfigs, encrypted_wallet: &[u8]) -> VcxResult<()> {
    trace!("reconstitute_restored_wallet >>> recovery_config: {:?}", secret!(recovery_config));

    let recovery_config = _write_tmp_encrypted_wallet_for_import(recovery_config, encrypted_wallet)?;

    info!("Deleting temporary wallet before the recovered wallet is imported");
    wallet::delete_wallet(&settings::get_config_value(settings::CONFIG_WALLET_NAME)?, None, None, None)?;

    import(&recovery_config.to_string()?)?;

    //Todo: Fix libindy
    // Deletes recovered encrypted wallet from the temporary location on the file system
    // This will be removed once libindy enables import/export without file system location
    let path = Path::new(&recovery_config.exported_wallet_path);
    fs::remove_file(path).map_err(|_| VcxError::from(VcxErrorKind::RetrieveExportedWallet))?;

    open_wallet(&recovery_config.wallet_name, None, None, None)?;

    trace!("reconstitute_restored_wallet <<<");

    Ok(())
}

fn _write_tmp_encrypted_wallet_for_import(recovery_config: &RestoreWalletConfigs, wallet: &[u8]) -> VcxResult<RestoreWalletConfigs> {
    trace!("_write_tmp_encrypted_wallet_for_import >>> recovery_config: {:?}", secret!(recovery_config));

    let tmp_dir = _unique_tmp_dir(&recovery_config.exported_wallet_path)?;

    if let Some(parent_path) = tmp_dir.parent() {
        fs::DirBuilder::new()
            .recursive(true)
            .create(parent_path).map_err(_io_err_res)?;
    }

    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp_dir).map_err(_io_err_res)?
        .write_all(wallet).map_err(_io_err_res)?;

    let mut new_path_config = recovery_config.clone();
    new_path_config.exported_wallet_path = tmp_dir.to_str().ok_or_else(|| _io_err_opt("Invalid unique temp directory"))?.to_string();

    trace!("_write_tmp_encrypted_wallet_for_import <<< new_path_config: {:?}", secret!(new_path_config));

    Ok(new_path_config)
}

fn _unique_tmp_dir(path_str: &str) -> VcxResult<PathBuf> {
    let path = PathBuf::from(path_str);
    let f_name = path.file_name().and_then(|os_str| os_str.to_str()).ok_or_else(|| _io_err_opt("Invalid file name"))?;
    Ok(Path::new(path_str).with_file_name(&format!("{}{}", rand::thread_rng().gen::<u32>(), f_name)))
}

fn _io_err_res(e: Error) -> VcxError { VcxError::from_msg(VcxErrorKind::IOError, format!("Wallet IO error: {:?}", e)) }

fn _io_err_opt(e: &str) -> VcxError { VcxError::from_msg(VcxErrorKind::IOError, format!("Wallet IO error: {}", e)) }

pub fn recover_dead_drop(vk: &str) -> VcxResult<CloudAddress> {
    trace!("recover_dead_drop >>> vk: {}", secret!(vk));
    info!("recovering dead drop");

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

    let cloud_addr = CloudAddress::from_str(&parse_message_from_response(&encrypted_ca)?)?;

    trace!("recover_dead_drop <<< cloud_addr: {:?}", secret!(cloud_addr));
    Ok(cloud_addr)
}

impl Handle<WalletBackup> {
    pub fn is_valid_handle(self) -> bool { WALLET_BACKUP_MAP.has_handle(self) }

    pub fn get_state(self) -> u32 {
        WALLET_BACKUP_MAP.get(self, |wb| {
            debug!("get state for wallet_backup {}", wb.get_source_id());
            Ok(wb.get_state().clone())
        }).unwrap_or(WalletBackupState::Uninitialized as u32)
    }

    pub fn get_source_id(self) -> VcxResult<String> {
        WALLET_BACKUP_MAP.get(self, |wb| {
            Ok(wb.get_source_id().clone())
        })
    }

    pub fn to_string(self) -> VcxResult<String> {
        WALLET_BACKUP_MAP.get(self, |obj| {
            WalletBackup::to_string(&obj)
        })
    }

    /*
       Todo: exported_wallet_path is needed because the only exposed libindy functionality for exporting
       an encrypted wallet, writes it to the file system. A possible better way is for libindy's export_wallet
       to optionally return an encrypted stream of bytes instead of writing it to the fs. This could also
       be done in a separate libindy api call if necessary.
       */
    pub fn backup_wallet(self, exported_wallet_path: &str) -> VcxResult<u32> {
        info!("backup_wallet >>> handle: {}, export_path: {}", self, secret!(exported_wallet_path));
        WALLET_BACKUP_MAP.get_mut(self, |wb| {
            wb.backup(exported_wallet_path)
        })
    }

    pub fn set_state(self, state: WalletBackupState) -> VcxResult<()> {
        WALLET_BACKUP_MAP.get_mut(self, |wb| {
            Ok(wb.set_state(state))
        })
    }

    pub fn update_state(self, message: Option<Message>) -> VcxResult<u32> {
        info!("update_state >>> source_id {}", self.get_source_id()?);
        WALLET_BACKUP_MAP.get_mut(self, move |wb| {
            wb.update_state(message)
        })
    }

    pub fn has_known_cloud_backup(self) -> bool {
        WALLET_BACKUP_MAP.get(self, |wb| {
            Ok(wb.has_stored_backup().clone())
        }).unwrap_or(false)
    }
}

pub fn from_string(wallet_backup_data: &str) -> VcxResult<Handle<WalletBackup>> {
    let wallet_backup: WalletBackup = WalletBackup::from_str(wallet_backup_data)?;

    let new_handle = WALLET_BACKUP_MAP.add(wallet_backup)?;

    info!("inserting handle {} into wallet backup table", new_handle);

    Ok(new_handle)
}
#[cfg(all(test, feature = "wallet_backup"))]
pub mod tests {
    use super::*;
    use serde_json::Value;
    use std::thread;
    use std::time::Duration;
    use utils::libindy::wallet;
    use std::fs::File;
    use utils::devsetup::*;

    pub const WALLET_PROVISION_AGENT_RESPONSE: &'static [u8; 2] = &[79, 75];
    static SOURCE_ID: &str = r#"12345"#;
    static FILE_PATH: &str = r#"/tmp/tmp_wallet"#;
    pub static BACKUP_KEY: &str = r#"8dvfYSt5d1taSd6yJdpjq4emkwsPDDLYxkNFysFD2cZY"#;
    pub static RECORD_TYPE: &str = r#"cloudBackupType"#;
    pub static ID: &str = r#"cloudBackupId"#;
    pub static RECORD_VALUE: &str = r#"save before cloud backup"#;
    pub static PATH: &str = r#"/tmp/cloud_backup.zip"#;

    fn backup_key_gen() -> String {
        base64::encode(&rand::thread_rng()
            .gen_ascii_chars()
            .take(32)
            .collect::<String>())
    }

    pub struct TestBackupData {
        pub wb_handle: Handle<WalletBackup>,
        pub recovery_vk: String,
        pub dd_address: String,
        pub locator: String,
        pub encryption_key: String,
        pub cloud_address: Vec<u8>,
        pub sig: Vec<u8>,
    }

    impl TestBackupData {
        pub fn new(handle: Option<Handle<WalletBackup>>, vk: Option<String>, dd_address: Option<String>,
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

    pub fn restore_config(path: Option<String>, backup_key: Option<String>) -> RestoreWalletConfigs {
        RestoreWalletConfigs {
            wallet_name: ::settings::get_wallet_name().unwrap(),
            wallet_key: settings::DEFAULT_WALLET_KEY.to_string(),
            exported_wallet_path: path.unwrap_or(PATH.to_string()),
            backup_key: backup_key.unwrap_or(BACKUP_KEY.to_string()),
            key_derivation: None,
        }
    }

    pub fn init_backup() -> TestBackupData {
        let backup_key = backup_key_gen();
        let mut wb = WalletBackup::create(SOURCE_ID, &backup_key).unwrap();
        wb.init_backup().unwrap();

        thread::sleep(Duration::from_secs(2));
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
                            Some(backup_key.to_string()),
        )
    }

    pub fn backup_wallet_utils() -> TestBackupData {
        wallet::add_record(RECORD_TYPE, ID, RECORD_VALUE, None).unwrap();
        let wb = init_backup();

        wb.wb_handle.backup_wallet(PATH).unwrap();
        thread::sleep(Duration::from_millis(1000));

        wb
    }

    #[cfg(all(feature = "agency", feature = "pool_tests"))]
    mod create_wallet_backup {
        use super::*;

        #[test]
        fn create_backup_succeeds_real() {
            let _setup = SetupConsumer::init();
            assert!(create_wallet_backup(SOURCE_ID, &backup_key_gen()).is_ok());
        }

        #[test]
        fn create_two_backup_init_succeeds_real() {
            let _setup = SetupConsumer::init();

            let backup_key = backup_key_gen();
            assert!(create_wallet_backup(SOURCE_ID, &backup_key).is_ok());
            assert!(create_wallet_backup(SOURCE_ID, &backup_key).is_ok());
        }
    }

    mod update_state {
        use super::*;

        #[test]
        fn update_state_success() {
            let _setup = SetupMocks::init();

            ::utils::httpclient::AgencyMock::set_next_response(WALLET_PROVISION_AGENT_RESPONSE);

            let handle = create_wallet_backup(SOURCE_ID, &backup_key_gen()).unwrap();
            assert!(handle.update_state(None).is_ok());
            assert_eq!(handle.get_state(), WalletBackupState::InitRequested as u32);
        }

        #[cfg(all(feature = "agency", feature = "pool_tests"))]
        #[test]
        fn update_state_with_provisioned_msg_changes_state_to_ready_to_export() {
            let _setup = SetupConsumer::init();

            let handle = create_wallet_backup(SOURCE_ID, backup_key_gen().as_str()).unwrap();
            thread::sleep(Duration::from_millis(2000));

            assert!(handle.update_state(None).is_ok());
            assert_eq!(handle.get_state(), WalletBackupState::ReadyToExportWallet as u32);
        }

        #[cfg(all(feature = "agency", feature = "pool_tests"))]
        #[test]
        fn update_state_with_backup_ack_msg_changes_state_to_ready_to_export() {
            let _setup = SetupConsumer::init();

            let handle = create_wallet_backup(SOURCE_ID, backup_key_gen().as_str()).unwrap();
            thread::sleep(Duration::from_millis(2000));

            assert!(handle.update_state(None).is_ok());
            assert_eq!(handle.get_state(), WalletBackupState::ReadyToExportWallet as u32);

            handle.backup_wallet(FILE_PATH).unwrap();
            assert_eq!(handle.get_state(), WalletBackupState::BackupInProgress as u32);

            thread::sleep(Duration::from_millis(2000));

            assert!(handle.update_state(None).is_ok());
            assert_eq!(handle.get_state(), WalletBackupState::ReadyToExportWallet as u32);
        }
    }

    mod serialization {
        use super::*;

        #[test]
        fn to_string_test() {
            let _setup = SetupMocks::init();

            ::utils::httpclient::AgencyMock::set_next_response(WALLET_PROVISION_AGENT_RESPONSE);

            let handle = create_wallet_backup(SOURCE_ID, backup_key_gen().as_str()).unwrap();
            let serialized = handle.to_string().unwrap();
            let j: Value = serde_json::from_str(&serialized).unwrap();
            assert_eq!(j["version"], "1.0");
            WalletBackup::from_str(&serialized).unwrap();
        }

        #[test]
        fn test_deserialize_fails() {
            assert_eq!(from_string("{}").unwrap_err().kind(), VcxErrorKind::InvalidJson);
        }
    }

    mod backup_wallet {
        use super::*;

        #[test]
        fn retrieving_exported_wallet_data_successful() {
            let _setup = SetupLibraryWallet::init();

            let data = _read_exported_wallet(backup_key_gen().as_str(), FILE_PATH);

            assert!(data.unwrap().len() > 0);
        }

        #[test]
        fn retrieve_exported_wallet_success_with_file_already_created() {
            let _setup = SetupLibraryWallet::init();

            File::create(FILE_PATH).and_then(|mut f| f.write_all(&[1, 2, 3])).unwrap();

            assert!(_read_exported_wallet(backup_key_gen().as_str(), FILE_PATH).is_ok());
        }

        #[test]
        fn backup_wallet_fails_with_invalid_handle() {
            let _setup = SetupMocks::init();
            assert_eq!(Handle::<WalletBackup>::dummy().backup_wallet(FILE_PATH).unwrap_err().kind(), VcxErrorKind::InvalidHandle)
        }

        #[cfg(all(feature = "agency", feature = "pool_tests"))]
        #[test]
        fn backup_wallet_succeeds_real() {
            let _setup = SetupConsumer::init();

            let five_mb = 4800000;
            let buf = vec![0x41u8; five_mb];
            let buf_str = ::std::str::from_utf8(&buf).unwrap();
            add_record("whatever", "bigbyte", buf_str, None).unwrap();
            let wallet_backup = create_wallet_backup(SOURCE_ID, &backup_key_gen()).unwrap();
            thread::sleep(Duration::from_millis(2000));

            assert_eq!(wallet_backup.get_state(), WalletBackupState::InitRequested as u32);
            assert!(wallet_backup.update_state(None).is_ok());

            wallet_backup.backup_wallet(FILE_PATH).unwrap();
            thread::sleep(Duration::from_millis(2000));
            assert_eq!(wallet_backup.get_state(), WalletBackupState::BackupInProgress as u32);

            thread::sleep(Duration::from_millis(2000));

            assert!(wallet_backup.update_state(None).is_ok());
            assert_eq!(wallet_backup.get_state(), WalletBackupState::ReadyToExportWallet as u32);
            assert!(wallet_backup.has_known_cloud_backup());
        }

        #[cfg(all(feature = "agency", feature = "pool_tests"))]
        #[test]
        fn backup_wallet_fails_when_size_is_over_max() {
            let _setup = SetupConsumer::init();

            let buf = vec![0x41u8; MAX_WALLET_SIZE as usize];
            let buf_str = ::std::str::from_utf8(&buf).unwrap();
            add_record("whatever", "bigbyte", buf_str, None).unwrap();
            let wallet_backup = create_wallet_backup(SOURCE_ID, &backup_key_gen()).unwrap();
            thread::sleep(Duration::from_millis(2000));

            assert_eq!(wallet_backup.get_state(), WalletBackupState::InitRequested as u32);
            assert!(wallet_backup.update_state(None).is_ok());

            let rc = wallet_backup.backup_wallet(FILE_PATH);
            assert_eq!(
                rc.unwrap_err().to_string(),
                "Error: Cloud Backup exceeds max size limit\n  Caused by: Cloud Backup exceeds max size limit\n"
            );
        }

        #[cfg(all(feature = "agency", feature = "pool_tests"))]
        #[test]
        fn backup_wallet_multiple_times_real() {
            let _setup = SetupConsumer::init();

            let wallet_backup = create_wallet_backup(SOURCE_ID, backup_key_gen().as_str()).unwrap();
            thread::sleep(Duration::from_millis(2000));

            assert_eq!(wallet_backup.get_state(), WalletBackupState::InitRequested as u32);
            assert!(wallet_backup.update_state(None).is_ok());

            wallet_backup.backup_wallet(FILE_PATH).unwrap();
            assert_eq!(wallet_backup.get_state(), WalletBackupState::BackupInProgress as u32);

            thread::sleep(Duration::from_millis(2000));

            assert!(wallet_backup.update_state(None).is_ok());
            assert_eq!(wallet_backup.get_state(), WalletBackupState::ReadyToExportWallet as u32);
            assert!(wallet_backup.has_known_cloud_backup());

            wallet_backup.backup_wallet(FILE_PATH).unwrap();
            assert_eq!(wallet_backup.get_state(), WalletBackupState::BackupInProgress as u32);
        }
    }

    #[cfg(all(feature = "agency", feature = "pool_tests"))]
    mod restore_wallet {
        use super::*;

        #[test]
        fn restore_wallet_real() {
            let wb;

            {
                let _setup = SetupConsumer::init();

                wb = backup_wallet_utils();
            }

            let _setup = SetupConsumer::init();

            let restore_config_str = restore_config(None, Some(wb.encryption_key)).to_string().unwrap();
            thread::sleep(Duration::from_secs(5));
            restore_wallet(&restore_config_str).unwrap();

            let options = json!({
                "retrieveType": true,
                "retrieveValue": true,
                "retrieveTags": true
            }).to_string();
            let record = wallet::get_record(RECORD_TYPE, ID, &options).unwrap();
            let record: serde_json::Value = serde_json::from_str(&record).unwrap();
            assert_eq!(&record, &json!({"value":RECORD_VALUE, "type": RECORD_TYPE, "id": ID, "tags": {}}));
        }

        #[test]
        fn restore_wallet_fails_with_no_backup() {
            let wb;
            {
                let _setup = SetupConsumer::init();

                wallet::add_record(RECORD_TYPE, ID, RECORD_VALUE, None).unwrap();
                wb = init_backup();
                thread::sleep(Duration::from_millis(2000));
            }

            let _setup = SetupConsumer::init();

            let rc = restore_wallet(&restore_config(None, Some(wb.encryption_key)).to_string().unwrap());
            assert_eq!(
                rc.unwrap_err().to_string(),
                "Error: Message failed in post\n  Caused by: Sending POST HTTP request failed with: {\"statusCode\":\"GNR-111\",\"statusMsg\":\"No Wallet Backup available to download\"}\n"
            );
        }

        #[cfg(feature = "too_long_request")]
        #[test]
        fn recovery_creates_file_structure_for_undefined_path_recovery_success() {
            let wb;
            {
                let _setup = SetupConsumer::init();

                wb = backup_wallet_utils();
            }

            let _setup = SetupConsumer::init();

            // Just to make sure path doesn't exist
            let base = "/tmp/uncreated/";
            let uncreated_path = format!("{}/nested/test.txt", base);
            ::std::fs::remove_dir_all(base).unwrap_or(println!("No Directory to delete before test"));
            let rc = restore_wallet(&restore_config(Some(uncreated_path.to_string()), Some(wb.encryption_key)).to_string().unwrap());
            ::std::fs::remove_dir_all(base).unwrap_or(println!("No Directory to delete after test"));

            assert!(rc.is_ok());
        }

        #[test]
        fn recovery_overwrites_export_path_when_file_already_exists() {
            let wb;
            {
                let _setup = SetupConsumer::init();
                wb = backup_wallet_utils();
            }

            let _setup = SetupConsumer::init();

            let base = "/tmp/existing/";
            let existing_file = format!("{}/test.txt", base);

            let wallet_name = ::settings::get_wallet_name().unwrap();

            let recovery_config = RestoreWalletConfigs {
                wallet_name: wallet_name.clone(),
                wallet_key: settings::get_config_value(::settings::CONFIG_WALLET_KEY).unwrap(),
                exported_wallet_path: existing_file,
                backup_key: settings::get_config_value(::settings::CONFIG_WALLET_BACKUP_KEY).unwrap_or(backup_key_gen()),
                key_derivation: None,
            };
            _write_tmp_encrypted_wallet_for_import(&recovery_config, &[1, 2, 3, 4, 5, 6, 7]).unwrap();

            restore_wallet(&restore_config(Some(recovery_config.exported_wallet_path.to_string()), Some(wb.encryption_key)).to_string().unwrap()).unwrap();
            ::std::fs::remove_dir_all(&PathBuf::from(&recovery_config.exported_wallet_path).parent().unwrap()).unwrap_or(println!("No Directory to delete after test"));
        }
    }

    #[cfg(all(feature = "agency", feature = "pool_tests", feature = "too_long_request"))]
    #[test]
    fn recovery_full() {
        let config_wallet_key = "CONFIG_WALLET_KEY";
        let wb;
        let consumer_wallet_name;
        let original_config;
        {
            let _setup = SetupConsumer::init();

            // 1.  Provision 1st time (Provision Async) + (Init)
            consumer_wallet_name = ::settings::get_wallet_name().unwrap();

            // 2. Insert test data into non secret portion of the wallet (config data)
            original_config = settings::tests::config_json();
            wallet::add_record(config_wallet_key, config_wallet_key, &original_config, None).unwrap();
            
            let five_mb = 4800000;
            let buf = vec![0x41u8; five_mb];
            let buf_str = ::std::str::from_utf8(&buf).unwrap();
            add_record("whatever", "bigbyte2", buf_str, None).unwrap();

            // 3. Backup Wallet
            wb = init_backup();
            wb.wb_handle.backup_wallet(PATH).unwrap();
            thread::sleep(Duration::from_millis(1000));
        }
        // 4. Destroy Environment

        {
            // 5. Provision 2nd time (for interaction with agency to get dead-drop)
            let _setup = SetupConsumer::init();

            // 6. Restore
            restore_wallet(&restore_config(None, Some(wb.encryption_key)).to_string().unwrap()).unwrap();
            thread::sleep(Duration::from_millis(1000));
        }

        // 7. Shutdown to clear configuration and close wallet

        // 8. Initialize with wallet info (simple init)
        ::settings::set_config_value(::settings::CONFIG_WALLET_KEY, ::settings::DEFAULT_WALLET_KEY);
        ::settings::set_config_value(::settings::CONFIG_WALLET_KEY_DERIVATION, ::settings::DEFAULT_WALLET_KEY_DERIVATION);
        ::settings::set_config_value(::settings::CONFIG_WALLET_NAME, &consumer_wallet_name);
        ::settings::set_config_value(::settings::CONFIG_WALLET_BACKUP_KEY, backup_key_gen().as_str());

        open_wallet(&consumer_wallet_name, None, None, None).unwrap();

        // 9. retrieve config data
        let options = json!( {
                "retrieveType": false,
                "retrieveValue": true,
                "retrieveTags": false
            }).to_string();
        let record = serde_json::from_str::<serde_json::Value>(&wallet::get_record(config_wallet_key, config_wallet_key, &options).unwrap()).unwrap();
        let retrieved_config_p1: String = serde_json::from_value(record.get("value").unwrap().to_owned()).unwrap();
        assert_eq!(&retrieved_config_p1, &original_config);

        // 10. shutdown to clear config
        ::api::vcx::vcx_shutdown(false);

        // 11. Full init with previously stored config
        crate::settings::process_config_string(&retrieved_config_p1, true).unwrap();
        open_wallet(&consumer_wallet_name, None, None, None).unwrap();

        let record = serde_json::from_str::<serde_json::Value>(&wallet::get_record(config_wallet_key, config_wallet_key, &options).unwrap()).unwrap();
        let retrieved_config_p2: String = serde_json::from_value(record.get("value").unwrap().to_owned()).unwrap();
        assert_eq!(&retrieved_config_p2, &original_config);
    }
}

