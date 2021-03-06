#[macro_use]
pub mod utils;
pub mod handlers;
pub mod messages;

pub const SERIALIZE_VERSION: &'static str = "2.0";

#[cfg(test)]
pub mod test {
    use core::fmt::Debug;
    use serde::Serialize;
    use crate::disclosed_proof::DisclosedProofs;
    use crate::issuer_credential::IssuerCredentials;
    use crate::utils::object_cache::Handle;
    use crate::connection::Connections;
    use crate::proof::Proofs;
    use rand;
    use crate::credential::Credentials;
    use crate::credential_def::CredentialDef;
    use crate::schema::CreateSchema;
    use rand::Rng;
    use crate::utils::devsetup::*;
    use crate::utils::libindy::wallet::*;
    use vdrtools_sys::WalletHandle;
    use crate::agent::messages::payload::PayloadV1;
    use crate::api::VcxStateType;
    use crate::aries::handlers::connection::types::OutofbandMeta;

    use crate::utils::libindy::anoncreds::holder::Holder;
    use crate::utils::libindy::anoncreds::types::CredentialInfo;
    use crate::agent::provisioning;

    pub fn source_id() -> String {
        String::from("test source id")
    }

    pub mod setup {
        use crate::settings::{CONFIG_WALLET_KEY_DERIVATION, DEFAULT_WALLET_KEY};
        use vdrtools_sys::WalletHandle;

        pub fn base_config() -> ::serde_json::Value {
            json!({
                "agency_did":"VsKV7grR1BUE29mG2Fm2kX",
                "agency_endpoint":"http://localhost:8080",
                "agency_verkey":"Hezce2UWMZ3wUhVkh2LfKSs8nDzWwzs2Win7EzNN3YaR",
                "institution_did":"V4SGRU86Z58d6TV7PBUe6f",
                "institution_logo_url":"<CHANGE_ME>",
                "institution_name":"<CHANGE_ME>",
                "institution_verkey":"GJ1SzoWzavQYfNL9XkaJdrQejfztN4XqdsiV4ct3LXKL",
                "protocol_type":"3.0",
                "remote_to_sdk_did":"LjC6xZPeYPeL5AjuRByMDA",
                "remote_to_sdk_verkey":"Bkd9WFmCydMCvLKL8x47qyQTN1nbyQ8rUK8JTsQRtLGE",
                "sdk_to_remote_did":"Mi3bbeWQDVpQCmGFBqWeYa",
                "sdk_to_remote_verkey":"CHcPnSn48wfrUhekmcFZAmx8NvhHCh72J73WToNiK9EX",
                "wallet_key":DEFAULT_WALLET_KEY,
                "wallet_name":"test_wallet",
                CONFIG_WALLET_KEY_DERIVATION:"RAW",
            })
        }

        pub struct AgencyModeSetup {
            pub wallet_name: String,
            pub wallet_handle: WalletHandle,
        }

        impl AgencyModeSetup {
            pub fn init() -> AgencyModeSetup {
                let wallet_name = "wallet_name";

                let mut config = base_config();
                config["wallet_name"] = json!(wallet_name);
                config["enable_test_mode"] = json!("true");

                crate::settings::process_config_string(&config.to_string(), false).unwrap();

                crate::utils::libindy::wallet::create_wallet(wallet_name, None, None, None).unwrap();
                let config = crate::utils::devsetup::config_with_wallet_handle(wallet_name, &config.to_string());

                crate::settings::process_config_string(&config.to_string(), false).unwrap();

                AgencyModeSetup {
                    wallet_name: wallet_name.to_string(),
                    wallet_handle: crate::utils::libindy::wallet::get_wallet_handle(),
                }
            }
        }

        impl Drop for AgencyModeSetup {
            fn drop(&mut self) {
                crate::utils::libindy::wallet::delete_wallet(&self.wallet_name, None, None, None).unwrap();
            }
        }
    }

    pub struct Pool {}

    impl Pool {
        pub fn open() -> Pool {
            crate::utils::libindy::vdr::tests::open_test_pool();
            Pool {}
        }
    }

    impl Drop for Pool {
        fn drop(&mut self) {
            crate::utils::libindy::vdr::close_vdr().unwrap();
            crate::utils::libindy::vdr::tests::delete_test_pool();
        }
    }

    #[derive(Debug)]
    pub struct Message {
        uid: String,
        message: String,
    }

    fn download_message(did: String, type_: &str) -> Message {
        let mut messages = crate::agent::messages::get_message::download_messages(Some(vec![did]), Some(vec![String::from("MS-103")]), None).unwrap();
        assert_eq!(1, messages.len());
        let messages = messages.pop().unwrap();

        for message in  messages.msgs.into_iter(){
            let payload: PayloadV1 = serde_json::from_str(&message.decrypted_payload.clone().unwrap()).unwrap();
            if payload.type_.name == type_ {
                return Message{
                    uid: message.uid,
                    message: payload.msg
                }
            }
        }
        panic!("Message not found")
    }

    pub struct Faber {
        pub wallet_name: String,
        pub wallet_handle: WalletHandle,
        pub connection_handle: Handle<Connections>,
        pub config: String,
        pub schema_handle: Handle<CreateSchema>,
        pub cred_def_handle: Handle<CredentialDef>,
        pub credential_handle: Handle<IssuerCredentials>,
        pub presentation_handle: Handle<Proofs>,
    }

    impl Faber {
        pub fn setup() -> Faber {
            crate::settings::clear_config();
            let wallet_name = "faber_wallet";

            let config = json!({
                "agency_url": AGENCY_ENDPOINT,
                "agency_did": AGENCY_DID,
                "agency_verkey": AGENCY_VERKEY,
                "wallet_name": wallet_name,
                "wallet_key": "123",
                "payment_method": "null",
                "enterprise_seed": "000000000000000000000000Trustee1",
                "protocol_type": "3.0",
            }).to_string();

            let config = provisioning::provision(&config).unwrap();

            let config = config_with_wallet_handle(wallet_name, &config);

            Faber {
                config,
                wallet_name: wallet_name.to_string(),
                schema_handle: Handle::dummy(),
                cred_def_handle: Handle::dummy(),
                connection_handle: Handle::dummy(),
                wallet_handle: get_wallet_handle(),
                credential_handle: Handle::dummy(),
                presentation_handle: Handle::dummy(),
            }
        }

        pub fn activate(&self) {
            crate::settings::clear_config();
            crate::settings::process_config_string(&self.config, false).unwrap();
            set_wallet_handle(self.wallet_handle);
        }

        pub fn send_message<T: Serialize + Debug>(&self, message: &T) {
            self.activate();
            let agent_info = self.connection_handle.get_completed_connection().unwrap();
            agent_info.agent.send_message(message, &agent_info.data.did_doc).unwrap();
        }

        pub fn create_schema(&mut self) {
            self.activate();
            let did = String::from("V4SGRU86Z58d6TV7PBUe6f");
            let data = r#"["name","date","degree", "empty_param"]"#.to_string();
            let name: String = rand::thread_rng().gen_ascii_chars().take(25).collect::<String>();
            let version: String = String::from("1.0");

            self.schema_handle = crate::schema::create_and_publish_schema("test_schema", did.clone(), name, version, data).unwrap();
        }

        pub fn create_credential_definition(&mut self) {
            self.activate();

            let schema_id = self.schema_handle.get_schema_id().unwrap();
            let did = String::from("V4SGRU86Z58d6TV7PBUe6f");
            let name = String::from("degree");
            let tag = String::from("tag");

            self.cred_def_handle = crate::credential_def::create_and_publish_credentialdef(String::from("test_cred_def"), name, did.clone(), schema_id, tag, String::from("{}")).unwrap();
        }

        pub fn create_presentation_request(&self) -> Handle<Proofs> {
            let requested_attrs = json!([
                {"name": "name"},
                {"name": "date"},
                {"name": "degree"},
                {"name": "empty_param", "restrictions": {"attr::empty_param::value": ""}}
            ]).to_string();

            crate::proof::create_proof(String::from("alice_degree"),
                                  requested_attrs,
                                  json!([]).to_string(),
                                  json!({}).to_string(),
                                  String::from("proof_from_alice")).unwrap()
        }

        pub fn create_invite(&mut self) -> String {
            self.activate();
            self.connection_handle = crate::connection::create_connection("alice").unwrap();
            self.connection_handle.connect(None).unwrap();
            self.connection_handle.update_state(None).unwrap();
            assert_eq!(2, self.connection_handle.get_state());

            self.connection_handle.get_invite_details(false).unwrap()
        }

        pub fn create_outofband_connection(&mut self, invite: OutofbandMeta) -> String {
            self.activate();

            self.connection_handle = crate::connection::create_outofband_connection("alice", invite.goal_code, invite.goal, invite.handshake, invite.request_attach).unwrap();
            self.connection_handle.connect(None).unwrap();
            self.connection_handle.get_invite_details(false).unwrap()
        }

        pub fn update_state(&self, expected_state: u32) {
            self.activate();
            self.connection_handle.update_state(None).unwrap();
            assert_eq!(expected_state, self.connection_handle.get_state());
        }

        pub fn ping(&self) {
            self.activate();
            self.connection_handle.send_ping(None).unwrap();
        }

        pub fn discovery_features(&self) {
            self.activate();
            self.connection_handle.send_discovery_features(None, None).unwrap();
        }

        pub fn connection_info(&self) -> ::serde_json::Value {
            self.activate();
            let details = self.connection_handle.get_connection_info().unwrap();
            ::serde_json::from_str(&details).unwrap()
        }

        pub fn offer_credential(&mut self) {
            self.activate();

            let did = String::from("V4SGRU86Z58d6TV7PBUe6f");
            let credential_data = json!({
                "name": "alice",
                "date": "05-2018",
                "degree": "maths",
                "empty_param": ""
            }).to_string();

            self.credential_handle = crate::issuer_credential::issuer_credential_create(self.cred_def_handle,
                                                                                   String::from("alice_degree"),
                                                                                   did,
                                                                                   String::from("cred"),
                                                                                   credential_data,
                                                                                   0).unwrap();
            self.credential_handle.send_credential_offer(self.connection_handle).unwrap();
            self.credential_handle.update_state(None).unwrap();
            assert_eq!(2, self.credential_handle.get_state().unwrap());
        }

        pub fn send_credential(&self) {
            self.activate();
            self.credential_handle.update_state(None).unwrap();
            assert_eq!(3, self.credential_handle.get_state().unwrap());

            self.credential_handle.send_credential(self.connection_handle).unwrap();
            self.credential_handle.update_state(None).unwrap();
            assert_eq!(VcxStateType::VcxStateAccepted as u32, self.credential_handle.get_state().unwrap());
        }

        pub fn request_presentation(&mut self) {
            self.activate();
            self.presentation_handle = self.create_presentation_request();
            assert_eq!(1, self.presentation_handle.get_state().unwrap());

            self.presentation_handle.send_proof_request(self.connection_handle).unwrap();
            self.presentation_handle.update_state(None).unwrap();

            assert_eq!(2, self.presentation_handle.get_state().unwrap());
        }

        pub fn verify_presentation(&self) {
            self.activate();
            self.update_proof_state(4)
        }

        pub fn update_proof_state(&self, expected_state: u32) {
            self.activate();

            self.presentation_handle.update_state(None).unwrap();
            assert_eq!(expected_state, self.presentation_handle.get_state().unwrap());
        }

        pub fn update_message(&self, uid: &str) {
            self.activate();
            let agent_info = self.connection_handle.get_completed_connection().unwrap();
            agent_info.agent.update_message_status(uid.to_string(), None).unwrap();
        }

        pub fn teardown(&self) {
            self.activate();
            close_wallet().unwrap();
            delete_wallet(&self.wallet_name, None, None, None).unwrap();
        }
    }

    pub struct Alice {
        pub wallet_name: String,
        pub wallet_handle: WalletHandle,
        pub connection_handle: Handle<Connections>,
        pub config: String,
        pub credential_handle: Handle<Credentials>,
        pub presentation_handle: Handle<DisclosedProofs>,
    }

    impl Alice {
        pub fn setup() -> Alice {
            crate::settings::clear_config();
            let wallet_name = "alice_wallet";

            let config = json!({
                "agency_url": C_AGENCY_ENDPOINT,
                "agency_did": C_AGENCY_DID,
                "agency_verkey": C_AGENCY_VERKEY,
                "wallet_name": wallet_name,
                "wallet_key": "123",
                "payment_method": "null",
                "protocol_type": "3.0",
            }).to_string();

            let config = provisioning::provision(&config).unwrap();

            let config = config_with_wallet_handle(&wallet_name, &config);

            Alice {
                config,
                wallet_name: wallet_name.to_string(),
                wallet_handle: get_wallet_handle(),
                connection_handle: Handle::dummy(),
                credential_handle: Handle::dummy(),
                presentation_handle: Handle::dummy(),
            }
        }

        pub fn activate(&self) {
            crate::settings::clear_config();
            crate::settings::process_config_string(&self.config, false).unwrap();
            set_wallet_handle(self.wallet_handle);
        }

        pub fn accept_invite(&mut self, invite: &str) {
            self.activate();
            self.connection_handle = crate::connection::create_connection_with_invite("faber", invite).unwrap();
            self.connection_handle.connect(None).unwrap();
            self.connection_handle.update_state(None).unwrap();
            assert_eq!(3, self.connection_handle.get_state());
        }

        pub fn accept_outofband_invite(&mut self, invite: &str) {
            self.activate();
            self.connection_handle = crate::connection::create_connection_with_outofband_invite("faber", invite).unwrap();
            self.connection_handle.connect(None).unwrap();
        }

        pub fn update_state(&self, expected_state: u32) {
            self.activate();
            self.connection_handle.update_state(None).unwrap();
            assert_eq!(expected_state, self.connection_handle.get_state());
        }

        pub fn download_message(&self, message_type: &str) -> Message {
            self.activate();
            let did = self.connection_handle.get_pw_did().unwrap();
            download_message(did, message_type)
        }

        pub fn accept_offer(&mut self) {
            self.activate();
            let offers = crate::credential::get_credential_offer_messages(self.connection_handle).unwrap();
            let offer = ::serde_json::from_str::<Vec<::serde_json::Value>>(&offers).unwrap()[0].clone();
            let offer_json = ::serde_json::to_string(&offer).unwrap();

            self.credential_handle = crate::credential::credential_create_with_offer("degree", &offer_json).unwrap();
            assert_eq!(3, self.credential_handle.get_state().unwrap());

            self.credential_handle.send_credential_request(self.connection_handle).unwrap();
            assert_eq!(2, self.credential_handle.get_state().unwrap());
        }

        pub fn accept_credential(&self) {
            self.activate();
            self.credential_handle.update_state(None).unwrap();
            assert_eq!(VcxStateType::VcxStateAccepted as u32, self.credential_handle.get_state().unwrap());
        }

        pub fn get_proof_request_messages(&self) -> String {
            self.activate();
            let presentation_requests = crate::disclosed_proof::get_proof_request_messages(self.connection_handle, None).unwrap();
            let presentation_request = ::serde_json::from_str::<Vec<::serde_json::Value>>(&presentation_requests).unwrap()[0].clone();
            let presentation_request_json = ::serde_json::to_string(&presentation_request).unwrap();
            presentation_request_json
        }

        pub fn get_credentials(&self) -> Vec<CredentialInfo> {
            self.activate();
            Holder::get_credentials().unwrap()
        }

        pub fn get_credentials_for_presentation(&self) -> serde_json::Value {
            self.activate();

            let credentials = self.presentation_handle.retrieve_credentials().unwrap();
            let credentials: ::std::collections::HashMap<String, ::serde_json::Value> = ::serde_json::from_str(&credentials).unwrap();

            let mut use_credentials = json!({});

            for (referent, credentials) in credentials["attrs"].as_object().unwrap().iter() {
                use_credentials["attrs"][referent] = json!({
                    "credential": credentials[0]
                })
            }

            use_credentials
        }

        pub fn send_presentation(&mut self) {
            self.activate();
            let presentation_request_json = self.get_proof_request_messages();

            self.presentation_handle = crate::disclosed_proof::create_proof("degree", &presentation_request_json).unwrap();

            let credentials = self.get_credentials_for_presentation();

            self.presentation_handle.generate_proof(credentials.to_string(), String::from("{}")).unwrap();
            assert_eq!(3, self.presentation_handle.get_state().unwrap());

            self.presentation_handle.send_proof(self.connection_handle).unwrap();
            assert_eq!(2, self.presentation_handle.get_state().unwrap());
        }

        pub fn decline_presentation_request(&mut self) {
            self.activate();
            let presentation_request_json = self.get_proof_request_messages();

            self.presentation_handle = crate::disclosed_proof::create_proof("degree", &presentation_request_json).unwrap();
            self.presentation_handle.decline_presentation_request(self.connection_handle, Some(String::from("reason")), None).unwrap();
        }

        pub fn propose_presentation(&mut self) {
            self.activate();
            let presentation_request_json = self.get_proof_request_messages();

            self.presentation_handle = crate::disclosed_proof::create_proof("degree", &presentation_request_json).unwrap();
            let proposal_data = json!({
                "attributes": [
                    {
                        "name": "first name"
                    }
                ],
                "predicates": [
                    {
                        "name": "age",
                        "predicate": ">",
                        "threshold": 18
                    }
                ]
            });
            self.presentation_handle.decline_presentation_request(self.connection_handle, None, Some(proposal_data.to_string())).unwrap();
        }

        pub fn ensure_presentation_verified(&self) {
            self.activate();
            self.presentation_handle.update_state(None).unwrap();
            assert_eq!(VcxStateType::VcxStateAccepted as u32, self.presentation_handle.get_state().unwrap());
        }

        pub fn update_message_status(&self, uid: String) {
            self.activate();
            let agent_info = self.connection_handle.get_completed_connection().unwrap();
            agent_info.agent.update_message_status(uid, None).unwrap();
        }

        pub fn delete_credential(&self) {
            self.activate();
            self.credential_handle.delete_credential().unwrap();
            assert_eq!(VcxStateType::VcxStateAccepted as u32, self.presentation_handle.get_state().unwrap());
        }
    }

    impl Drop for Faber {
        fn drop(&mut self) {
            self.activate();
            close_wallet().unwrap();
            delete_wallet(&self.wallet_name, None, None, None).unwrap();
        }
    }

    impl Drop for Alice {
        fn drop(&mut self) {
            self.activate();
            close_wallet().unwrap();
            delete_wallet(&self.wallet_name, None, None, None).unwrap();
        }
    }

    #[cfg(feature = "aries")]
    mod aries {
        use super::*;
        #[test]
        fn aries_demo() {
            let _pool = Pool::open();

            let mut faber = Faber::setup();
            let mut alice = Alice::setup();

            // Publish Schema and Credential Definition
            faber.create_schema();

            ::std::thread::sleep(::std::time::Duration::from_secs(2));

            faber.create_credential_definition();

            // Connection
            let invite = faber.create_invite();
            alice.accept_invite(&invite);

            faber.update_state(3);
            alice.update_state(4);
            faber.update_state(4);

            // Credential issuance
            faber.offer_credential();
            alice.accept_offer();
            faber.send_credential();
            alice.accept_credential();
            assert_eq!(1, alice.get_credentials().len());

            // Credential Presentation
            faber.request_presentation();
            alice.send_presentation();
            faber.verify_presentation();
            alice.ensure_presentation_verified();

            // Alice delete credential
            alice.delete_credential();
            assert_eq!(0, alice.get_credentials().len());
        }

        #[test]
        fn aries_demo_handle_connection_related_messages() {
            let _pool = Pool::open();

            let mut faber = Faber::setup();
            let mut alice = Alice::setup();

            // Publish Schema and Credential Definition
            faber.create_schema();

            ::std::thread::sleep(::std::time::Duration::from_secs(2));

            faber.create_credential_definition();

            // Connection
            let invite = faber.create_invite();
            alice.accept_invite(&invite);

            faber.update_state(3);
            alice.update_state(4);
            faber.update_state(4);

            // Ping
            faber.ping();

            alice.update_state(4);

            faber.update_state(4);

            let faber_connection_info = faber.connection_info();
            assert!(faber_connection_info["their"]["protocols"].as_array().is_none());

            // Discovery Features
            faber.discovery_features();

            alice.update_state(4);

            faber.update_state(4);

            let faber_connection_info = faber.connection_info();
            assert!(faber_connection_info["their"]["protocols"].as_array().unwrap().len() > 0);
        }

        #[test]
        fn aries_demo_create_with_message_id_flow() {
            let _pool = Pool::open();

            let mut faber = Faber::setup();
            let mut alice = Alice::setup();

            // Publish Schema and Credential Definition
            faber.create_schema();

            ::std::thread::sleep(::std::time::Duration::from_secs(2));

            faber.create_credential_definition();

            // Connection
            let invite = faber.create_invite();
            alice.accept_invite(&invite);

            faber.update_state(3);
            alice.update_state(4);
            faber.update_state(4);

            /*
             Create with message id flow
            */

            // Credential issuance
            faber.offer_credential();

            // Alice creates Credential object with message id
            {
                let message = alice.download_message("credential-offer");
                let (credential_handle, _credential_offer) = crate::credential::credential_create_with_msgid("test", alice.connection_handle, &message.uid).unwrap();
                alice.credential_handle = credential_handle;

                alice.credential_handle.send_credential_request(alice.connection_handle).unwrap();
                assert_eq!(2, alice.credential_handle.get_state().unwrap());
            }

            faber.send_credential();
            alice.accept_credential();

            // Credential Presentation
            faber.request_presentation();

            // Alice creates Presentation object with message id
            {
                let message = alice.download_message("presentation-request");
                let (presentation_handle, _presentation_request) = crate::disclosed_proof::create_proof_with_msgid("test", alice.connection_handle, &message.uid).unwrap();
                alice.presentation_handle = presentation_handle;

                let credentials = alice.get_credentials_for_presentation();

                alice.presentation_handle.generate_proof(credentials.to_string(), String::from("{}")).unwrap();
                assert_eq!(3, alice.presentation_handle.get_state().unwrap());

                alice.presentation_handle.send_proof(alice.connection_handle).unwrap();
                assert_eq!(2, alice.presentation_handle.get_state().unwrap());
            }

            faber.verify_presentation();
        }

        #[test]
        fn aries_demo_download_message_flow() {
            let _pool = Pool::open();

            let mut faber = Faber::setup();
            let mut alice = Alice::setup();

            // Publish Schema and Credential Definition
            faber.create_schema();

            ::std::thread::sleep(::std::time::Duration::from_secs(2));

            faber.create_credential_definition();

            // Connection
            let invite = faber.create_invite();
            alice.accept_invite(&invite);

            faber.update_state(3);
            alice.update_state(4);
            faber.update_state(4);

            /*
             Create with message flow
            */

            // Credential issuance
            faber.offer_credential();

            // Alice creates Credential object with Offer
            {
                let message = alice.download_message("credential-offer");

                alice.credential_handle = crate::credential::credential_create_with_offer("test", &message.message).unwrap();

                alice.update_message_status(message.uid);

                alice.credential_handle.send_credential_request(alice.connection_handle).unwrap();
                assert_eq!(2, alice.credential_handle.get_state().unwrap());
            }

            faber.send_credential();
            alice.accept_credential();

            // Credential Presentation
            faber.request_presentation();

            // Alice creates Presentation object with Proof Request
            {
                let message = alice.download_message("presentation-request");

                alice.presentation_handle = crate::disclosed_proof::create_proof("test", &message.message).unwrap();

                alice.update_message_status(message.uid);

                let credentials = alice.get_credentials_for_presentation();

                alice.presentation_handle.generate_proof(credentials.to_string(), String::from("{}")).unwrap();
                assert_eq!(3, alice.presentation_handle.get_state().unwrap());

                alice.presentation_handle.send_proof(alice.connection_handle).unwrap();
                assert_eq!(2, alice.presentation_handle.get_state().unwrap());
            }

            faber.verify_presentation();
        }

        use crate::aries::messages::outofband::invitation::Invitation as OutofbandInvitation;
        use crate::aries::messages::a2a::A2AMessage;

        #[test]
        fn test_outofband_connection_works() {
            let _pool = Pool::open();

            let mut faber = Faber::setup();
            let mut alice = Alice::setup();

            // Publish Schema and Credential Definition
            faber.create_schema();

            ::std::thread::sleep(::std::time::Duration::from_secs(2));

            faber.create_credential_definition();

            let meta = OutofbandMeta {
                goal_code: None,
                goal: Some(String::from("Test Goal")),
                handshake: true,
                request_attach: None
            };
            let invite = faber.create_outofband_connection(meta);
            println!("invite {}", invite);
            let outofband_invite: OutofbandInvitation = ::serde_json::from_str(&invite).unwrap();
            assert_eq!(1, outofband_invite.handshake_protocols().len());
            assert_eq!(0, outofband_invite.requests_attach().0.len());

            alice.accept_outofband_invite(&invite);

            // Connection is not completed
            assert_eq!(VcxStateType::VcxStateOfferSent as u32, faber.connection_handle.get_state());
            assert_eq!(VcxStateType::VcxStateRequestReceived as u32, alice.connection_handle.get_state());

            // Complete connection
            faber.update_state(3);
            alice.update_state(4);
            faber.update_state(4);

            // Credential issuance
            faber.offer_credential();
            alice.accept_offer();
            faber.send_credential();
            alice.accept_credential();
        }

        #[test]
        fn test_outofband_connection_works_without_handshake() {
            let _pool = Pool::open();

            let mut faber = Faber::setup();
            let mut alice = Alice::setup();

            // Publish Schema and Credential Definition
            faber.create_schema();

            ::std::thread::sleep(::std::time::Duration::from_secs(2));

            faber.create_credential_definition();

            let meta = OutofbandMeta {
                goal_code: None,
                goal: Some(String::from("Test Goal")),
                handshake: false,
                request_attach: Some(String::from("{}"))
            };
            let invite = faber.create_outofband_connection(meta);
            let outofband_invite: OutofbandInvitation = ::serde_json::from_str(&invite).unwrap();
            assert_eq!(0, outofband_invite.handshake_protocols().len());
            assert_eq!(1, outofband_invite.requests_attach().0.len());

            alice.accept_outofband_invite(&invite);

            // Connection is not completed
            assert_eq!(VcxStateType::VcxStateAccepted as u32, faber.connection_handle.get_state());
            assert_eq!(VcxStateType::VcxStateAccepted as u32, alice.connection_handle.get_state());

            // Alice Send Basic Message
            alice.activate();

            {
                let basic_message = r#"Hi there"#;
                alice.connection_handle.send_generic_message(basic_message, "").unwrap();

                faber.activate();
                let messages = faber.connection_handle.get_messages().unwrap();
                assert_eq!(1, messages.len());

                let message = messages.values().next().unwrap().clone();

                match message {
                    A2AMessage::BasicMessage(message) => assert_eq!(basic_message, message.content),
                    _ => assert!(false)
                }
            }
        }
    }
}
