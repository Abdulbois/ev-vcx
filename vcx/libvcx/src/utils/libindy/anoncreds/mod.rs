pub mod issuer;
pub mod holder;
pub mod verifier;
pub mod blob_storage;
pub mod utils;
pub mod types;

use futures::Future;
use crate::indy::anoncreds;

use crate::error::prelude::*;

pub fn libindy_to_unqualified(entity: &str) -> VcxResult<String> {
    anoncreds::to_unqualified(entity)
        .wait()
        .map_err(VcxError::from)
}

#[cfg(test)]
pub mod tests {
    use rand::Rng;
    use std::thread;
    use std::time::Duration;

    use crate::utils::object_cache::Handle;
    use crate::credential_def::CredentialDef;
    use crate::settings;
    use crate::utils::get_temp_dir_path;
    use crate::utils::constants::*;
    use crate::utils::devsetup::*;
    use crate::utils::random::random_number;
    use crate::utils::libindy::ledger;
    use crate::utils::libindy::ledger::request::Request;
    use crate::utils::libindy::ledger::utils::TxnTypes;
    use crate::utils::libindy::anoncreds::{
        issuer::Issuer,
        holder::Holder,
    };
    use crate::utils::libindy::ledger::query::Query;

    pub fn create_schema(attr_list: &str) -> (String, String) {
        let data = attr_list.to_string();
        let schema_name: String = rand::thread_rng().gen_ascii_chars().take(25).collect::<String>();
        let schema_version: String = format!("{}.{}", random_number().to_string(), random_number().to_string());
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        Issuer::create_schema(&institution_did, &schema_name, &schema_version, &data).unwrap()
    }

    pub fn create_schema_req(schema_json: &str) -> String {
        let request = Request::schema(schema_json).unwrap();
        Request::append_txn_author_agreement(&request).unwrap()
    }

    pub fn create_and_write_test_schema(attr_list: &str) -> (String, String) {
        let (schema_id, schema_json) = create_schema(attr_list);
        ledger::utils::sign_and_submit_txn(&schema_json,TxnTypes::Schema).unwrap();
        thread::sleep(Duration::from_millis(1000));
        (schema_id, schema_json)
    }

    pub fn create_and_store_credential_def(attr_list: &str, support_rev: bool) -> (String, String, String, String, Handle<CredentialDef>, Option<String>) {
        /* create schema */
        let (schema_id, schema_json) = create_and_write_test_schema(attr_list);

        let name: String = rand::thread_rng().gen_ascii_chars().take(25).collect::<String>();
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

        /* create cred-def */
        let mut revocation_details = json!({"support_revocation":support_rev});
        if support_rev {
            revocation_details["tails_file"] = json!(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string());
            revocation_details["max_creds"] = json!(10);
        }
        let handle = crate::credential_def::create_and_publish_credentialdef("1".to_string(),
                                                                             name,
                                                                             institution_did.clone(),
                                                                             schema_id.clone(),
                                                                             "tag1".to_string(),
                                                                             revocation_details.to_string()).unwrap();

        thread::sleep(Duration::from_millis(1000));
        let cred_def_id = handle.get_cred_def_id().unwrap();
        thread::sleep(Duration::from_millis(1000));
        let (_, cred_def_json) = ledger::query::Query::get_cred_def(&cred_def_id).unwrap();
        let rev_reg_id = handle.get_rev_reg_id().unwrap();
        (schema_id, schema_json, cred_def_id, cred_def_json, handle, rev_reg_id)
    }

    pub fn create_credential_offer(attr_list: &str, revocation: bool) -> (String, String, String, String, String, Option<String>) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, _, rev_reg_id) = create_and_store_credential_def(attr_list, revocation);

        let offer = Issuer::create_credential_offer(&cred_def_id).unwrap();
        (schema_id, schema_json, cred_def_id, cred_def_json, offer, rev_reg_id)
    }

    pub fn create_credential_req(attr_list: &str, revocation: bool) -> (String, String, String, String, String, String, String, Option<String>) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, offer, rev_reg_id) = create_credential_offer(attr_list, revocation);
        let institution_did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (req, req_meta) = Holder::create_credential_req(&institution_did, &offer, &cred_def_json).unwrap();
        (schema_id, schema_json, cred_def_id, cred_def_json, offer, req, req_meta, rev_reg_id)
    }

    pub fn create_and_store_credential(attr_list: &str, revocation: bool) -> (String, String, String, String, String, String, String, String, Option<String>, Option<String>) {
        let (schema_id, schema_json, cred_def_id, cred_def_json, offer, req, req_meta, rev_reg_id) = create_credential_req(attr_list, revocation);

        /* create cred */
        let credential_data = r#"{"address1": ["123 Main St"], "address2": ["Suite 3"], "city": ["Draper"], "state": ["UT"], "zip": ["84000"]}"#;
        let encoded_attributes = crate::issuer_credential::encode_attributes(&credential_data).unwrap();
        let (_, tails_file) = if revocation {
            let (_id, json) = Query::get_rev_reg_def(&rev_reg_id.clone().unwrap()).unwrap();
            (Some(json), Some(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap().to_string().to_string()))
        } else { (None, None) };

        let (cred, cred_rev_id, _) = Issuer::create_credential(&offer, &req, &encoded_attributes, rev_reg_id.as_deref(), tails_file.as_deref()).unwrap();
        /* store cred */
        let cred_id = crate::utils::libindy::anoncreds::holder::Holder::store_credential(None, &req_meta, &cred, &cred_def_json).unwrap();
        (schema_id, schema_json, cred_def_id, cred_def_json, offer, req, req_meta, cred_id, rev_reg_id, cred_rev_id)
    }

    pub fn create_proof() -> (String, String, String, String) {
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id, _, _)
            = create_and_store_credential(crate::utils::constants::DEFAULT_SCHEMA_ATTRS, false);

        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "zip_2": json!({
                   "name":"zip",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "self_attest_3": json!({
                   "name":"self_attest",
               }),
           }),
           "requested_predicates": json!({}),
        }).to_string();

        let requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true},
                 "zip_2": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{}
        }).to_string();

        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
        let schemas = json!({
            schema_id: schema_json,
        }).to_string();

        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let cred_defs = json!({
            cred_def_id: cred_def_json,
        }).to_string();

        Holder::get_credentials_for_proof_req(&proof_req).unwrap();

        let proof = Holder::create_proof(
            &proof_req,
            &requested_credentials_json,
            "main",
            &schemas,
            &cred_defs,
            None).unwrap();
        (schemas, cred_defs, proof_req, proof)
    }

    pub fn create_self_attested_proof() -> (String, String) {
        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
               }),
               "zip_2": json!({
                   "name":"zip",
               }),
           }),
           "requested_predicates": json!({}),
        }).to_string();

        let requested_credentials_json = json!({
              "self_attested_attributes":{
                 "address1_1": "my_self_attested_address",
                 "zip_2": "my_self_attested_zip"
              },
              "requested_attributes":{},
              "requested_predicates":{}
        }).to_string();

        let schemas = json!({}).to_string();
        let cred_defs = json!({}).to_string();

        Holder::get_credentials_for_proof_req(&proof_req).unwrap();

        let proof = Holder::create_proof(
            &proof_req,
            &requested_credentials_json,
            "main",
            &schemas,
            &cred_defs,
            None).unwrap();
        (proof_req, proof)
    }

    pub fn create_proof_with_predicate(include_predicate_cred: bool) -> (String, String, String, String) {
        let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
        let (schema_id, schema_json, cred_def_id, cred_def_json, _offer, _req, _req_meta, cred_id, _, _)
            = create_and_store_credential(crate::utils::constants::DEFAULT_SCHEMA_ATTRS, false);

        let proof_req = json!({
           "nonce":"123432421212",
           "name":"proof_req_1",
           "version":"0.1",
           "requested_attributes": json!({
               "address1_1": json!({
                   "name":"address1",
                   "restrictions": [json!({ "issuer_did": did })]
               }),
               "self_attest_3": json!({
                   "name":"self_attest",
               }),
           }),
           "requested_predicates": json!({
               "zip_3": {"name":"zip", "p_type":">=", "p_value":18}
           }),
        }).to_string();

        let requested_credentials_json;
        if include_predicate_cred {
            requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{
                  "zip_3": {"cred_id": cred_id}
              }
            }).to_string();
        } else {
            requested_credentials_json = json!({
              "self_attested_attributes":{
                 "self_attest_3": "my_self_attested_val"
              },
              "requested_attributes":{
                 "address1_1": {"cred_id": cred_id, "revealed": true}
                },
              "requested_predicates":{
              }
            }).to_string();
        }

        let schema_json: serde_json::Value = serde_json::from_str(&schema_json).unwrap();
        let schemas = json!({
            schema_id: schema_json,
        }).to_string();

        let cred_def_json: serde_json::Value = serde_json::from_str(&cred_def_json).unwrap();
        let cred_defs = json!({
            cred_def_id: cred_def_json,
        }).to_string();

        Holder::get_credentials_for_proof_req(&proof_req).unwrap();

        let proof = Holder::create_proof(
            &proof_req,
            &requested_credentials_json,
            "main",
            &schemas,
            &cred_defs,
            None).unwrap();
        (schemas, cred_defs, proof_req, proof)
    }

    #[test]
    fn test_create_cred_def() {
        let _setup = SetupMocks::init();

        let (id, _) = Issuer::create_and_store_credential_def("did", SCHEMAS_JSON, "tag_1", None, Some(false)).unwrap();
        assert_eq!(id, CRED_DEF_ID);
    }

    #[test]
    fn from_ledger_schema_id() {
        let _setup = SetupMocks::init();

        let (id, retrieved_schema) = Query::get_schema(SCHEMA_ID).unwrap();
        assert_eq!(&retrieved_schema, SCHEMA_JSON);
        assert_eq!(&id, SCHEMA_ID);
    }

    #[cfg(feature = "pool_tests")]
    mod pool_tests {
        use super::*;
        use crate::utils::constants::TEST_TAILS_FILE;
        use crate::utils::libindy::anoncreds::verifier::Verifier;
        use crate::error::VcxErrorKind;

        #[test]
        fn test_prover_verify_proof() {
            let _setup = SetupLibraryWalletPool::init();

            let (schemas, cred_defs, proof_req, proof) = create_proof();

            let proof_validation = Verifier::verify_proof(
                &proof_req,
                &proof,
                &schemas,
                &cred_defs,
                "{}",
                "{}",
            ).unwrap();

            assert!(proof_validation);
        }

        #[test]
        fn test_prover_verify_proof_with_predicate_success_case() {
            let _setup = SetupLibraryWalletPool::init();

            let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(true);

            let proof_validation = Verifier::verify_proof(
                &proof_req,
                &proof,
                &schemas,
                &cred_defs,
                "{}",
                "{}",
            ).unwrap();

            assert!(proof_validation);
        }

        #[test]
        fn test_prover_verify_proof_with_predicate_fail_case() {
            let _setup = SetupLibraryWalletPool::init();

            let (schemas, cred_defs, proof_req, proof) = create_proof_with_predicate(false);

            Verifier::verify_proof(
                &proof_req,
                &proof,
                &schemas,
                &cred_defs,
                "{}",
                "{}",
            ).unwrap_err();
        }

        #[test]
        fn tests_libindy_prover_get_credentials() {
            let _setup = SetupLibraryWallet::init();

            let proof_req = "{";
            let result = Holder::get_credentials_for_proof_req(&proof_req);
            assert_eq!(result.unwrap_err().kind(), VcxErrorKind::InvalidProofRequest);

            let proof_req = json!({
                "nonce":"123432421212",
                "name":"proof_req_1",
                "version":"0.1",
                "requested_attributes": json!({
                    "address1_1": json!({
                        "name":"address1",
                    }),
                    "zip_2": json!({
                        "name":"zip",
                    }),
                }),
                "requested_predicates": json!({}),
            }).to_string();
            let _result = Holder::get_credentials_for_proof_req(&proof_req).unwrap();

            let result_malformed_json = Holder::get_credentials_for_proof_req("{}").unwrap_err();
            assert_eq!(result_malformed_json.kind(), VcxErrorKind::InvalidProofRequest);
        }

        #[test]
        fn test_issuer_revoke_credential() {
            let _setup = SetupLibraryWalletPool::init();

            let rc = Issuer::revoke_credential(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap(), "", "");
            assert!(rc.is_err());

            let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id)
                = create_and_store_credential(crate::utils::constants::DEFAULT_SCHEMA_ATTRS, true);
            let rc = Issuer::revoke_credential(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap(), &rev_reg_id.unwrap(), &cred_rev_id.unwrap());

            assert!(rc.is_ok());
        }

        #[test]
        fn test_create_cred_def_real() {
            let _setup = SetupLibraryWalletPool::init();

            let (schema_id, _) = create_and_write_test_schema(crate::utils::constants::DEFAULT_SCHEMA_ATTRS);
            let (_, schema_json) = ledger::query::Query::get_schema(&schema_id).unwrap();
            let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

            let (_, cred_def_json) = Issuer::create_and_store_credential_def(&did, &schema_json, "tag_1", None, Some(true)).unwrap();
            ledger::utils::publish_cred_def(&cred_def_json).unwrap();
        }

        #[test]
        fn test_rev_reg_def_fails_for_cred_def_created_without_revocation() {
            let _setup = SetupLibraryWalletPool::init();

            // Cred def is created with support_revocation=false,
            // revoc_reg_def will fail in libindy because cred_Def doesn't have revocation keys
            let (_, _, cred_def_id, _, _, _) = create_and_store_credential_def(crate::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
            let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();
            let rc = Issuer::create_and_store_revoc_reg(&did, &cred_def_id, get_temp_dir_path("path.txt").to_str().unwrap(), 2);

            assert_eq!(rc.unwrap_err().kind(), VcxErrorKind::LibindyInvalidStructure);
        }

        #[test]
        fn test_create_rev_reg_def() {
            let _setup = SetupLibraryWalletPool::init();

            let (schema_id, _) = create_and_write_test_schema(crate::utils::constants::DEFAULT_SCHEMA_ATTRS);
            let (_, schema_json) = Query::get_schema(&schema_id).unwrap();
            let did = settings::get_config_value(settings::CONFIG_INSTITUTION_DID).unwrap();

            let (cred_def_id, cred_def_json) = Issuer::create_and_store_credential_def(&did, &schema_json, "tag_1", None, Some(true)).unwrap();
            ledger::utils::publish_cred_def(&cred_def_json).unwrap();
            let (rev_reg_def_id, rev_reg_def_json, rev_reg_entry_json) = Issuer::create_and_store_revoc_reg(&did, &cred_def_id, "tails.txt", 2).unwrap();
            ledger::utils::publish_rev_reg_def(&did, &rev_reg_def_json).unwrap();
            ledger::utils::publish_rev_reg_delta(&did, &rev_reg_def_id, &rev_reg_entry_json).unwrap();
        }

        #[test]
        fn test_get_rev_reg_def_json() {
            let _setup = SetupLibraryWalletPool::init();

            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, _, _, _, rev_reg_id) = create_and_store_credential_def(attrs, true);

            let rev_reg_id = rev_reg_id.unwrap();
            let (id, _json) = Query::get_rev_reg_def(&rev_reg_id).unwrap();
            assert_eq!(id, rev_reg_id);
        }

        #[test]
        fn test_get_rev_reg_delta_json() {
            let _setup = SetupLibraryWalletPool::init();

            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, _, _, _, rev_reg_id) = create_and_store_credential_def(attrs, true);
            let rev_reg_id = rev_reg_id.unwrap();

            let (id, _delta, _timestamp) = Query::get_rev_reg_delta(&rev_reg_id, None, None).unwrap();
            assert_eq!(id, rev_reg_id);
        }

        #[test]
        fn test_get_rev_reg() {
            let _setup = SetupLibraryWalletPool::init();

            let attrs = r#"["address1","address2","city","state","zip"]"#;
            let (_, _, _, _, _, rev_reg_id) =
                crate::utils::libindy::anoncreds::tests::create_and_store_credential_def(attrs, true);
            let rev_reg_id = rev_reg_id.unwrap();

            let (id, _rev_reg, _timestamp) = Query::get_rev_reg(&rev_reg_id, time::get_time().sec as u64).unwrap();
            assert_eq!(id, rev_reg_id);
        }

        #[test]
        fn from_pool_ledger_with_id() {
            let _setup = SetupLibraryWalletPool::init();

            let (schema_id, _schema_json) = create_and_write_test_schema(crate::utils::constants::DEFAULT_SCHEMA_ATTRS);

            let rc = Query::get_schema(&schema_id);

            let (_id, retrieved_schema) = rc.unwrap();
            assert!(retrieved_schema.contains(&schema_id));
        }

        #[test]
        fn test_revoke_credential() {
            let _setup = SetupLibraryWalletPool::init();

            let (_, _, _, _, _, _, _, _, rev_reg_id, cred_rev_id)
                = create_and_store_credential(crate::utils::constants::DEFAULT_SCHEMA_ATTRS, true);

            let rev_reg_id = rev_reg_id.unwrap();
            let (_, first_rev_reg_delta, first_timestamp) = Query::get_rev_reg_delta(&rev_reg_id, None, None).unwrap();
            let (_, test_same_delta, test_same_timestamp) = Query::get_rev_reg_delta(&rev_reg_id, None, None).unwrap();

            assert_eq!(first_rev_reg_delta, test_same_delta);
            assert_eq!(first_timestamp, test_same_timestamp);

            let _revoked_rev_reg_delta = Issuer::revoke_credential(get_temp_dir_path(TEST_TAILS_FILE).to_str().unwrap(), &rev_reg_id, cred_rev_id.unwrap().as_str()).unwrap();

            // Delta should change after revocation
            let (_, second_rev_reg_delta, _) = Query::get_rev_reg_delta(&rev_reg_id, Some(first_timestamp + 1), None).unwrap();

            assert_ne!(first_rev_reg_delta, second_rev_reg_delta);
        }

        #[test]
        fn test_fetch_public_entities() {
            let _setup = SetupLibraryWalletPool::init();

            let _ = create_and_store_credential(crate::utils::constants::DEFAULT_SCHEMA_ATTRS, false);
            Holder::fetch_public_entities().unwrap();
        }
    }
}
