use serde_json;

use crate::error::{VcxError, VcxErrorKind, VcxResult};
use crate::settings;
use crate::utils::libindy::ledger::types::TxnAuthorAgreement;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TxnAuthorAgreementAcceptanceData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taa_digest: Option<String>,
    pub acceptance_mechanism_type: String,
    pub time_of_acceptance: u64,
}

pub fn get_txn_author_agreement() -> VcxResult<Option<TxnAuthorAgreementAcceptanceData>> {
    match settings::get_config_value(settings::CONFIG_TXN_AUTHOR_AGREEMENT) {
        Ok(value) => {
            let meta: TxnAuthorAgreementAcceptanceData = serde_json::from_str(&value)
                .map_err(|err| VcxError::from_msg(VcxErrorKind::InvalidJson,
                                                  format!("Could not parse TxnAuthorAgreementAcceptanceData from JSON. Err: {:?}", err)))?;
            Ok(Some(meta))
        }
        Err(_) => Ok(None)
    }
}

impl Into<TxnAuthorAgreement> for TxnAuthorAgreementAcceptanceData {
    fn into(self) -> TxnAuthorAgreement {
        TxnAuthorAgreement {
            text: self.text,
            version: self.version,
            taa_digest: self.taa_digest,
            acc_mech_type: self.acceptance_mechanism_type,
            time: self.time_of_acceptance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::devsetup::SetupDefaults;

    const TEXT: &str = "indy agreement";
    const VERSION: &str = "1.0.0";
    const ACCEPTANCE_MECHANISM: &str = "acceptance mechanism label 1";
    const TIME_OF_ACCEPTANCE: u64 = 123456789;

    #[test]
    fn get_txn_author_agreement_works() {
        let _setup = SetupDefaults::init();

        let meta = TxnAuthorAgreementAcceptanceData {
            text: Some(TEXT.to_string()),
            version: Some(VERSION.to_string()),
            taa_digest: None,
            acceptance_mechanism_type: ACCEPTANCE_MECHANISM.to_string(),
            time_of_acceptance: TIME_OF_ACCEPTANCE,
        };
        settings::set_config_value(settings::CONFIG_TXN_AUTHOR_AGREEMENT, &json!(meta).to_string());

        let meta = get_txn_author_agreement().unwrap().unwrap();

        let expected_meta = TxnAuthorAgreementAcceptanceData {
            text: Some(TEXT.to_string()),
            version: Some(VERSION.to_string()),
            taa_digest: None,
            acceptance_mechanism_type: ACCEPTANCE_MECHANISM.to_string(),
            time_of_acceptance: TIME_OF_ACCEPTANCE,
        };

        assert_eq!(expected_meta, meta);
    }

    #[test]
    fn get_txn_author_agreement_works_for_not_set() {
        let _setup = SetupDefaults::init();

        assert!(get_txn_author_agreement().unwrap().is_none());
    }
}
