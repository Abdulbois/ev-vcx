use std::fs;
use std::sync::Once;
use crate::utils::{threadpool, get_temp_dir_path};
use crate::{settings, utils};
use crate::utils::libindy::wallet::{reset_wallet_handle, delete_wallet, create_wallet};
use crate::utils::libindy::vdr::tests::{open_test_pool, delete_test_pool};
use crate::agent::provisioning::agent_provisioning_v0_7;
use crate::utils::libindy::vdr::reset_vdr;
use crate::settings::set_defaults;
use crate::utils::libindy::crypto::sign;
use crate::utils::constants;
use crate::utils::libindy::wallet;
use crate::utils::object_cache::{ObjectCache, Handle};
use crate::indy::WalletHandle;
use crate::utils::libindy::wallet::init_wallet;
use crate::utils::plugins::init_plugin;
use crate::utils::file::write_file;
use crate::utils::logger::LibvcxDefaultLogger;
use crate::settings::wallet::get_wallet_name;
use crate::agent::provisioning;
use crate::utils::libindy::ledger::utils::TxnTypes;

pub struct SetupEmpty; // empty

pub struct SetupDefaults; // set default settings

pub struct SetupMocks; // set default settings and enable test mode

pub struct SetupAriesMocks; // set default settings, aries communication protocol and enable test mode

pub struct SetupIndyMocks; // set default settings and enable indy mode

pub struct SetupWallet; // set default settings and create indy wallet

pub struct SetupWalletAndPool; // set default settings and create indy wallet/ pool

pub struct SetupLibraryWallet; // set default settings and init indy wallet

pub struct SetupLibraryWalletPool; // set default settings, init indy wallet, init pool, set default fees

pub struct SetupLibraryWalletPoolZeroFees;  // set default settings, init indy wallet, init pool, set zero fees

pub struct SetupAgencyMock; // set default settings and enable mock agency mode

pub struct SetupLibraryAgencyV1; // init indy wallet, init pool, provision 2 agents. use protocol type 1.0

pub struct SetupLibraryAgencyV1ZeroFees; // init indy wallet, init pool, provision 2 agents. use protocol type 1.0, set zero fees

pub struct SetupLibraryAgencyV2; // init indy wallet, init pool, provision 2 agents. use protocol type 2.0

pub struct SetupLibraryAgencyV2ZeroFees; // init indy wallet, init pool, provision 2 agents. use protocol type 2.0, set zero fees

//TODO: This will be removed once libvcx only supports provisioning 0.7
pub struct SetupLibraryAgencyV2NewProvisioning;

// init indy wallet, init pool, provision 2 agents. use protocol type 2.0, set zero fees
pub struct SetupLibraryAgencyV2ZeroFeesNewProvisioning; // init indy wallet, init pool, provision 2 agents. use protocol type 2.0, set zero fees

pub struct SetupConsumer; // init indy wallet, init pool, provision 1 consumer agent, use protocol type 1.0

fn setup() {
    settings::clear_config();
    set_defaults();
    threadpool::init();
    init_test_logging();
}

fn tear_down() {
    settings::clear_config();
    reset_wallet_handle();
    reset_vdr();
}

impl SetupEmpty {
    pub fn init() {
        setup();
        settings::clear_config();
    }
}

impl Drop for SetupEmpty {
    fn drop(&mut self) {
        tear_down()
    }
}

impl SetupDefaults {
    pub fn init() {
        setup();
    }
}

impl Drop for SetupDefaults {
    fn drop(&mut self) {
        tear_down()
    }
}

impl SetupMocks {
    pub fn init() -> SetupMocks {
        setup();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "1.0");
        SetupMocks
    }
}

impl Drop for SetupMocks {
    fn drop(&mut self) {
        tear_down()
    }
}

impl SetupAriesMocks {
    pub fn init() -> SetupAriesMocks {
        setup();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "true");
        settings::set_config_value(settings::CONFIG_PROTOCOL_TYPE, "3.0");
        SetupAriesMocks
    }
}

impl Drop for SetupAriesMocks {
    fn drop(&mut self) {
        tear_down()
    }
}

impl SetupLibraryWallet {
    pub fn init() -> SetupLibraryWallet {
        setup();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
        init_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();
        SetupLibraryWallet
    }
}

impl Drop for SetupLibraryWallet {
    fn drop(&mut self) {
        delete_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();
        tear_down()
    }
}

impl SetupWallet {
    pub fn init() -> SetupWallet {
        setup();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
        create_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();
        SetupWallet
    }
}

impl Drop for SetupWallet {
    fn drop(&mut self) {
        delete_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();
        tear_down()
    }
}

impl SetupWalletAndPool {
    pub fn init() -> SetupWalletAndPool {
        setup();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
        create_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();
        SetupWalletAndPool
    }
}

impl Drop for SetupWalletAndPool {
    fn drop(&mut self) {
        delete_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();
        delete_test_pool();
        tear_down()
    }
}

impl SetupIndyMocks {
    pub fn init() -> SetupIndyMocks {
        setup();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "indy");
        init_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();
        SetupIndyMocks
    }
}

impl Drop for SetupIndyMocks {
    fn drop(&mut self) {
        tear_down()
    }
}

impl SetupLibraryWalletPool {
    pub fn init() -> SetupLibraryWalletPool {
        setup();
        setup_indy_env(false);
        SetupLibraryWalletPool
    }
}

impl Drop for SetupLibraryWalletPool {
    fn drop(&mut self) {
        cleanup_indy_env();
        tear_down()
    }
}

impl SetupLibraryWalletPoolZeroFees {
    pub fn init() -> SetupLibraryWalletPoolZeroFees {
        setup();
        setup_indy_env(true);
        SetupLibraryWalletPoolZeroFees
    }
}

impl Drop for SetupLibraryWalletPoolZeroFees {
    fn drop(&mut self) {
        cleanup_indy_env();
        tear_down()
    }
}

impl SetupAgencyMock {
    pub fn init() -> SetupAgencyMock {
        setup();
        settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "agency");
        init_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();
        SetupAgencyMock
    }
}

impl Drop for SetupAgencyMock {
    fn drop(&mut self) {
        delete_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();
        tear_down()
    }
}

impl SetupLibraryAgencyV1 {
    pub fn init() -> SetupLibraryAgencyV1 {
        setup();
        setup_agency_env("1.0", false);
        SetupLibraryAgencyV1
    }
}

impl Drop for SetupLibraryAgencyV1 {
    fn drop(&mut self) {
        cleanup_agency_env();
        tear_down()
    }
}

impl SetupLibraryAgencyV1ZeroFees {
    pub fn init() -> SetupLibraryAgencyV1ZeroFees {
        setup();
        setup_agency_env("1.0", true);
        SetupLibraryAgencyV1ZeroFees
    }
}

impl Drop for SetupLibraryAgencyV1ZeroFees {
    fn drop(&mut self) {
        cleanup_agency_env();
        tear_down()
    }
}

impl SetupLibraryAgencyV2NewProvisioning {
    pub fn init() -> SetupLibraryAgencyV2NewProvisioning {
        setup();
        setup_agency_env_new_protocol("2.0", false);
        SetupLibraryAgencyV2NewProvisioning
    }
}

impl Drop for SetupLibraryAgencyV2NewProvisioning {
    fn drop(&mut self) {
        cleanup_agency_env();
        tear_down()
    }
}

impl SetupLibraryAgencyV2ZeroFeesNewProvisioning {
    pub fn init() -> SetupLibraryAgencyV2ZeroFeesNewProvisioning {
        setup();
        setup_agency_env_new_protocol("2.0", true);
        SetupLibraryAgencyV2ZeroFeesNewProvisioning
    }
}

impl Drop for SetupLibraryAgencyV2ZeroFeesNewProvisioning {
    fn drop(&mut self) {
        cleanup_agency_env();
        tear_down()
    }
}

impl SetupLibraryAgencyV2 {
    pub fn init() -> SetupLibraryAgencyV2 {
        setup();
        setup_agency_env("2.0", false);
        SetupLibraryAgencyV2
    }
}

impl Drop for SetupLibraryAgencyV2 {
    fn drop(&mut self) {
        cleanup_agency_env();
        tear_down()
    }
}

impl SetupLibraryAgencyV2ZeroFees {
    pub fn init() -> SetupLibraryAgencyV2ZeroFees {
        setup();
        setup_agency_env("2.0", true);
        SetupLibraryAgencyV2ZeroFees
    }
}

impl Drop for SetupLibraryAgencyV2ZeroFees {
    fn drop(&mut self) {
        cleanup_agency_env();
        tear_down()
    }
}

impl SetupConsumer {
    pub fn init() -> SetupConsumer {
        setup();
        setup_consumer_env("1.0");
        SetupConsumer
    }
}

impl Drop for SetupConsumer {
    fn drop(&mut self) {
        cleanup_consumer_env();
        tear_down()
    }
}

#[macro_export]
macro_rules! assert_match {
    ($pattern:pat, $var:expr) => (
        assert!(match $var {
            $pattern => true,
            _ => false
        })
    );
}

static mut INSTITUTION_CONFIG: Handle<String> = Handle::dummy();
static mut CONSUMER_CONFIG: Handle<String> = Handle::dummy();

lazy_static! {
    static ref CONFIG_STRING: ObjectCache<String> = Default::default();
}

/* dev */
/*
pub const AGENCY_ENDPOINT: &'static str = "http://int-eas.pdev.evernym.com";
pub const AGENCY_DID: &'static str = "YRuVCckY6vfZfX9kcQZe3u";
pub const AGENCY_VERKEY: &'static str = "J8Yct6FwmarXjrE2khZesUXRVVSVczSoa9sFaGe6AD2v";

pub const C_AGENCY_ENDPOINT: &'static str = "http://int-agency.pdev.evernym.com";
pub const C_AGENCY_DID: &'static str = "dTLdJqRZLwMuWSogcKfBT";
pub const C_AGENCY_VERKEY: &'static str = "LsPQTDHi294TexkFmZK9Q9vW4YGtQRuLV8wuyZi94yH";
*/

/* sandbox */
/*pub const AGENCY_ENDPOINT: &'static str = "http://sbx-eas.pdev.evernym.com";
pub const AGENCY_DID: &'static str = "HB7qFQyFxx4ptjKqioEtd8";
pub const AGENCY_VERKEY: &'static str = "9pJkfHyfJMZjUjS7EZ2q2HX55CbFQPKpQ9eTjSAUMLU8";

pub const C_AGENCY_ENDPOINT: &'static str = "http://sbx-agency.pdev.evernym.com";
pub const C_AGENCY_DID: &'static str = "Nv9oqGX57gy15kPSJzo2i4";
pub const C_AGENCY_VERKEY: &'static str = "CwpcjCc6MtVNdQgwoonNMFoR6dhzmRXHHaUCRSrjh8gj";*/

/* Team2 */
/*
pub const AGENCY_ENDPOINT: &'static str = "https://eas-team2.pdev.evernym.com";
pub const AGENCY_DID: &'static str = "CV65RFpeCtPu82hNF9i61G";
pub const AGENCY_VERKEY: &'static str = "7G3LhXFKXKTMv7XGx1Qc9wqkMbwcU2iLBHL8x1JXWWC2";

pub const C_AGENCY_ENDPOINT: &'static str = "https://agency-team2.pdev.evernym.com";
pub const C_AGENCY_DID: &'static str = "TGLBMTcW9fHdkSqown9jD8";
pub const C_AGENCY_VERKEY: &'static str = "FKGV9jKvorzKPtPJPNLZkYPkLhiS1VbxdvBgd1RjcQHR";
 */

/* ci pipeline -- qa environment */
//pub const AGENCY_ENDPOINT: &'static str = "https://eas.pqa.evernym.com";
//pub const AGENCY_DID: &'static str = "QreyffsPPLCUqetQbahYNu";
//pub const AGENCY_VERKEY: &'static str = "E194CfHi5GGRiy1nThMorPf3jBEm4tvcAgcb65JFfxc7";
//
//pub const C_AGENCY_ENDPOINT: &'static str = "https://agency.pqa.evernym.com";
//pub const C_AGENCY_DID: &'static str = "LhiSANFohRXBWaKSZDvTH5";
//pub const C_AGENCY_VERKEY: &'static str = "BjpTLofEbVYJ8xxXQxScbmubHsgpHY5uvScfXqW9B1vB";

/* DEV RC */
pub const AGENCY_ENDPOINT: &'static str = "https://eas.pdev.evernym.com";
pub const AGENCY_DID: &'static str = "LTjTWsezEmV4wJYD5Ufxvk";
pub const AGENCY_VERKEY: &'static str = "BcCSmgdfChLqmtBkkA26YotWVFBNnyY45WCnQziF4cqN";

pub const C_AGENCY_ENDPOINT: &'static str = "https://agency.pdev.evernym.com";
pub const C_AGENCY_DID: &'static str = "LiLBGgFarh954ZtTByLM1C";
pub const C_AGENCY_VERKEY: &'static str = "Bk9wFrud3rz8v3nAFKGib6sQs8zHWzZxfst7Wh3Mbc9W";

/* DEV Team 1 */
// pub const AGENCY_ENDPOINT: &'static str = "https://eas-team1.pdev.evernym.com";
// pub const AGENCY_DID: &'static str = "CV65RFpeCtPu82hNF9i61G";
// pub const AGENCY_VERKEY: &'static str = "7G3LhXFKXKTMv7XGx1Qc9wqkMbwcU2iLBHL8x1JXWWC2";
//
// pub const C_AGENCY_ENDPOINT: &'static str = "https://agency-team1.pdev.evernym.com";
// pub const C_AGENCY_DID: &'static str = "TGLBMTcW9fHdkSqown9jD8";
// pub const C_AGENCY_VERKEY: &'static str = "FKGV9jKvorzKPtPJPNLZkYPkLhiS1VbxdvBgd1RjcQHR";

static TEST_LOGGING_INIT: Once = Once::new();

fn init_test_logging() {
    TEST_LOGGING_INIT.call_once(|| {
        LibvcxDefaultLogger::init(Some(String::from("debug"))).ok();
    })
}

pub fn create_new_seed() -> String {
    let x = rand::random::<u32>();
    format!("{:032}", x)
}

pub fn setup_indy_env(_use_zero_fees: bool) {
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");

    init_plugin(settings::DEFAULT_PAYMENT_PLUGIN, settings::DEFAULT_PAYMENT_INIT_FUNCTION);

    init_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();

    settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
    open_test_pool();

    crate::utils::libindy::anoncreds::holder::Holder::create_master_secret(settings::DEFAULT_LINK_SECRET_ALIAS).unwrap();

    let (my_did, my_vk) = crate::utils::libindy::crypto::create_and_store_my_did(Some(constants::TRUSTEE_SEED), None).unwrap();
    settings::set_config_value(settings::CONFIG_INSTITUTION_DID, &my_did);
    settings::set_config_value(settings::CONFIG_INSTITUTION_VERKEY, &my_vk);
}

pub fn cleanup_indy_env() {
    delete_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).ok();
    delete_test_pool();
}

pub fn cleanup_agency_env() {
    set_institution();
    delete_wallet(&get_wallet_name().unwrap(), None, None, None).unwrap();

    set_consumer();
    delete_wallet(&get_wallet_name().unwrap(), None, None, None).unwrap();

    delete_test_pool();
}

pub fn set_institution() {
    settings::clear_config();
    unsafe {
        CONFIG_STRING.get(INSTITUTION_CONFIG, |t| {
            settings::set_config_value(settings::CONFIG_PAYMENT_METHOD, settings::DEFAULT_PAYMENT_METHOD);
            settings::process_config_string(&t, true)
        }).unwrap();
    }
    change_wallet_handle();
}

pub fn set_consumer() {
    settings::clear_config();
    unsafe {
        CONFIG_STRING.get(CONSUMER_CONFIG, |t| {
            settings::set_config_value(settings::CONFIG_PAYMENT_METHOD, settings::DEFAULT_PAYMENT_METHOD);
            settings::process_config_string(&t, true)
        }).unwrap();
    }
    change_wallet_handle();
}

fn change_wallet_handle() {
    let wallet_handle = settings::get_config_value(settings::CONFIG_WALLET_HANDLE).unwrap();
    unsafe { wallet::WALLET_HANDLE = WalletHandle(wallet_handle.parse::<i32>().unwrap()) }
}

pub fn setup_agency_env(protocol_type: &str, _use_zero_fees: bool) {
    settings::clear_config();

    init_plugin(settings::DEFAULT_PAYMENT_PLUGIN, settings::DEFAULT_PAYMENT_INIT_FUNCTION);

    let enterprise_wallet_name = format!("{}_{}", constants::ENTERPRISE_PREFIX, settings::DEFAULT_WALLET_NAME);

    let seed1 = create_new_seed();
    let config = json!({
        "agency_url": AGENCY_ENDPOINT.to_string(),
        "agency_did": AGENCY_DID.to_string(),
        "agency_verkey": AGENCY_VERKEY.to_string(),
        "wallet_name": enterprise_wallet_name,
        "wallet_key": settings::DEFAULT_WALLET_KEY.to_string(),
        "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        "enterprise_seed": seed1,
        "agent_seed": seed1,
        "name": "institution".to_string(),
        "logo": "http://www.logo.com".to_string(),
        "path": constants::GENESIS_PATH.to_string(),
        "protocol_type": protocol_type,
    });

    let enterprise_config = provisioning::provision(&config.to_string()).unwrap();

    crate::api::vcx::vcx_shutdown(false);

    let consumer_wallet_name = format!("{}_{}", constants::CONSUMER_PREFIX, settings::DEFAULT_WALLET_NAME);
    let seed2 = create_new_seed();
    let config = json!({
        "agency_url": C_AGENCY_ENDPOINT.to_string(),
        "agency_did": C_AGENCY_DID.to_string(),
        "agency_verkey": C_AGENCY_VERKEY.to_string(),
        "wallet_name": consumer_wallet_name,
        "wallet_key": settings::DEFAULT_WALLET_KEY.to_string(),
        "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION.to_string(),
        "enterprise_seed": seed2,
        "agent_seed": seed2,
        "name": "consumer".to_string(),
        "logo": "http://www.logo.com".to_string(),
        "path": constants::GENESIS_PATH.to_string(),
        "protocol_type": protocol_type,
    });

    let consumer_config = provisioning::provision(&config.to_string()).unwrap();

    unsafe {
        INSTITUTION_CONFIG = CONFIG_STRING.add(config_with_wallet_handle(&enterprise_wallet_name, &enterprise_config)).unwrap();
    }
    unsafe {
        CONSUMER_CONFIG = CONFIG_STRING.add(config_with_wallet_handle(&consumer_wallet_name, &consumer_config.to_string())).unwrap();
    }
    settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
    open_test_pool();


    // grab the generated did and vk from the consumer and enterprise
    set_consumer();
    let did2 = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
    let vk2 = settings::get_config_value(settings::CONFIG_INSTITUTION_VERKEY).unwrap();
    set_institution();
    let did1 = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
    let vk1 = settings::get_config_value(settings::CONFIG_INSTITUTION_VERKEY).unwrap();
    settings::clear_config();

    // make enterprise and consumer trustees on the ledger
    wallet::init_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();

    let data = json!({
        "dest": did1,
        "verkey": vk1,
        "role": "TRUSTEE",
    }).to_string();
    crate::utils::libindy::ledger::utils::sign_and_submit_txn(&data, TxnTypes::DID).unwrap();

    let data = json!({
        "dest": did2,
        "verkey": vk2,
        "role": "TRUSTEE",
    }).to_string();
    crate::utils::libindy::ledger::utils::sign_and_submit_txn(&data, TxnTypes::DID).unwrap();
    wallet::delete_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();

    // as trustees, mint tokens into each wallet
    set_consumer();
}

pub fn sign_provision_token(keys: &str, nonce: &str, time: &str, sponsee_id: &str, sponsor_id: &str) -> String {
    let sig = sign(keys, &(format!("{}{}{}{}", nonce, time, sponsee_id, sponsor_id)).as_bytes()).unwrap();
    base64::encode(&sig)
}

//TODO: This will be removed once libvcx only supports provisioning 0.7
pub fn setup_agency_env_new_protocol(protocol_type: &str, _use_zero_fees: bool) {
    settings::clear_config();

    init_plugin(settings::DEFAULT_PAYMENT_PLUGIN, settings::DEFAULT_PAYMENT_INIT_FUNCTION);
    let sponsee_id = "id";
    let sponsor_id = "evernym-test-sponsorabc123";
    let nonce = "nonce";
    let time = chrono::offset::Utc::now().to_rfc3339();
    let enterprise_wallet_name = format!("{}_{}", crate::utils::constants::ENTERPRISE_PREFIX, settings::DEFAULT_WALLET_NAME);
    wallet::init_wallet(&enterprise_wallet_name, None, None, None).unwrap();
    let keys = crate::utils::libindy::crypto::create_key(Some("000000000000000000000000Trustee1")).unwrap();
    let sig = sign_provision_token(&keys, nonce, &time, sponsee_id, sponsor_id);
    wallet::close_wallet().err();
    wallet::delete_wallet(&enterprise_wallet_name, None, None, None).err();
    let test_token = json!( {
            "sponseeId": sponsee_id.to_string(),
            "sponsorId": sponsor_id.to_string(),
            "nonce": nonce.to_string(),
            "timestamp": time.to_string(),
            "sig": sig,
            "sponsorVerKey": keys.to_string()
        }).to_string();

    let enterprise_wallet_name = format!("{}_{}", constants::ENTERPRISE_PREFIX, settings::DEFAULT_WALLET_NAME);

    let seed1 = create_new_seed();
    let config = json!({
        "agency_url": AGENCY_ENDPOINT.to_string(),
        "agency_did": AGENCY_DID.to_string(),
        "agency_verkey": AGENCY_VERKEY.to_string(),
        "wallet_name": enterprise_wallet_name,
        "wallet_key": settings::DEFAULT_WALLET_KEY.to_string(),
        "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION,
        "enterprise_seed": seed1,
        "agent_seed": seed1,
        "name": "institution".to_string(),
        "logo": "http://www.logo.com".to_string(),
        "path": constants::GENESIS_PATH.to_string(),
        "protocol_type": protocol_type,
    });

    let enterprise_config = agent_provisioning_v0_7::provision(&config.to_string(), &test_token).unwrap();

    crate::api::vcx::vcx_shutdown(false);

    let consumer_wallet_name = format!("{}_{}", constants::CONSUMER_PREFIX, settings::DEFAULT_WALLET_NAME);
    let seed2 = create_new_seed();
    let config = json!({
        "agency_url": C_AGENCY_ENDPOINT.to_string(),
        "agency_did": C_AGENCY_DID.to_string(),
        "agency_verkey": C_AGENCY_VERKEY.to_string(),
        "wallet_name": consumer_wallet_name,
        "wallet_key": settings::DEFAULT_WALLET_KEY.to_string(),
        "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION.to_string(),
        "enterprise_seed": seed2,
        "agent_seed": seed2,
        "name": "consumer".to_string(),
        "logo": "http://www.logo.com".to_string(),
        "path": constants::GENESIS_PATH.to_string(),
        "protocol_type": protocol_type,
    });

    let consumer_config = agent_provisioning_v0_7::provision(&config.to_string(), &test_token).unwrap();

    unsafe {
        INSTITUTION_CONFIG = CONFIG_STRING.add(config_with_wallet_handle(&enterprise_wallet_name, &enterprise_config)).unwrap();
    }
    unsafe {
        CONSUMER_CONFIG = CONFIG_STRING.add(config_with_wallet_handle(&consumer_wallet_name, &consumer_config.to_string())).unwrap();
    }
    settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
    open_test_pool();


    // grab the generated did and vk from the consumer and enterprise
    set_consumer();
    let did2 = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
    let vk2 = settings::get_config_value(settings::CONFIG_INSTITUTION_VERKEY).unwrap();
    set_institution();
    let did1 = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
    let vk1 = settings::get_config_value(settings::CONFIG_INSTITUTION_VERKEY).unwrap();
    settings::clear_config();

    // make enterprise and consumer trustees on the ledger
    wallet::init_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();
    let data = json!({
        "dest": did1,
        "verkey": vk1,
        "role": "TRUSTEE",
    }).to_string();
    crate::utils::libindy::ledger::utils::sign_and_submit_txn(&data, TxnTypes::DID).unwrap();

    let data = json!({
        "dest": did2,
        "verkey": vk2,
        "role": "TRUSTEE",
    }).to_string();
    crate::utils::libindy::ledger::utils::sign_and_submit_txn(&data, TxnTypes::DID).unwrap();
    wallet::delete_wallet(settings::DEFAULT_WALLET_NAME, None, None, None).unwrap();

    set_institution();
}

pub fn config_with_wallet_handle(wallet_n: &str, config: &str) -> String {
    let wallet_handle = wallet::open_wallet(wallet_n, None, None, None).unwrap();
    let mut config: serde_json::Value = serde_json::from_str(config).unwrap();
    config[settings::CONFIG_WALLET_HANDLE] = json!(wallet_handle.0.to_string());
    config.to_string()
}

pub fn setup_wallet_env(test_name: &str) -> Result<WalletHandle, String> {
    settings::set_config_value(settings::CONFIG_ENABLE_TEST_MODE, "false");
    init_wallet(test_name, None, None, None).map_err(|e| format!("Unable to init_wallet in tests: {}", e))
}

pub fn cleanup_wallet_env(test_name: &str) -> Result<(), String> {
    delete_wallet(test_name, None, None, None).or(Err(format!("Unable to delete wallet: {}", test_name)))
}

pub fn setup_consumer_env(protocol_type: &str) {
    settings::clear_config();

    init_plugin(settings::DEFAULT_PAYMENT_PLUGIN, settings::DEFAULT_PAYMENT_INIT_FUNCTION);

    let consumer_wallet_name = format!("{}_{}", constants::CONSUMER_PREFIX, settings::DEFAULT_WALLET_NAME);
    let seed2 = create_new_seed();
    let config = json!({
        "agency_url": C_AGENCY_ENDPOINT.to_string(),
        "agency_did": C_AGENCY_DID.to_string(),
        "agency_verkey": C_AGENCY_VERKEY.to_string(),
        "wallet_name": consumer_wallet_name,
        "wallet_key": settings::DEFAULT_WALLET_KEY.to_string(),
        "wallet_key_derivation": settings::DEFAULT_WALLET_KEY_DERIVATION.to_string(),
        "enterprise_seed": seed2,
        "agent_seed": seed2,
        "name": "consumer".to_string(),
        "logo": "http://www.logo.com".to_string(),
        "path": constants::GENESIS_PATH.to_string(),
        "protocol_type": protocol_type,
    });

    let consumer_config = provisioning::provision(&config.to_string()).unwrap();

    unsafe {
        CONSUMER_CONFIG = CONFIG_STRING.add(config_with_wallet_handle(&consumer_wallet_name, &consumer_config.to_string())).unwrap();
    }
    settings::set_config_value(settings::CONFIG_GENESIS_PATH, utils::get_temp_dir_path(settings::DEFAULT_GENESIS_PATH).to_str().unwrap());
    open_test_pool();

    // grab the generated did and vk from the consumer and enterprise
    set_consumer();
}

pub fn cleanup_consumer_env() {
//    set_consumer();
    delete_wallet(&get_wallet_name().unwrap(), None, None, None).ok();
    delete_test_pool();
}

pub struct TempFile {
    pub path: String,
}

impl TempFile {
    pub fn prepare_path(filename: &str) -> TempFile {
        let file_path = get_temp_dir_path(filename).to_str().unwrap().to_string();
        TempFile { path: file_path }
    }

    pub fn create(filename: &str) -> TempFile {
        let file_path = get_temp_dir_path(filename).to_str().unwrap().to_string();
        fs::File::create(&file_path).unwrap();
        TempFile { path: file_path }
    }

    pub fn create_with_data(filename: &str, data: &str) -> TempFile {
        let mut file = TempFile::create(filename);
        file.write(data);
        file
    }

    pub fn write(&mut self, data: &str) {
        write_file(&self.path, data).unwrap()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        fs::remove_file(&self.path).unwrap()
    }
}

#[cfg(feature = "agency")]
#[cfg(feature = "pool_tests")]
mod tests {
    use super::*;

    #[cfg(feature = "agency")]
    #[cfg(feature = "pool_tests")]
    #[test]
    pub fn test_two_enterprise_connections() {
        let _setup = SetupLibraryAgencyV1ZeroFees::init();

        let (_faber, _alice) = crate::connection::tests::create_connected_connections();
        let (_faber, _alice) = crate::connection::tests::create_connected_connections();
    }
}
