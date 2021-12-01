use futures::Future;

use crate::indy::anoncreds;
use crate::error::prelude::*;

pub struct Verifier {}

impl Verifier {
    pub fn generate_nonce() -> VcxResult<String> {
        anoncreds::generate_nonce()
            .wait()
            .map_err(VcxError::from)
    }

    pub fn verify_proof(proof_req_json: &str,
                        proof_json: &str,
                        schemas_json: &str,
                        credential_defs_json: &str,
                        rev_reg_defs_json: &str,
                        rev_regs_json: &str) -> VcxResult<bool> {
        anoncreds::verifier_verify_proof(proof_req_json,
                                         proof_json,
                                         schemas_json,
                                         credential_defs_json,
                                         rev_reg_defs_json,
                                         rev_regs_json)
            .wait()
            .map_err(VcxError::from)
    }
}