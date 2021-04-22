extern crate zeroize;

use self::zeroize::Zeroize;
use std::fmt;

#[derive(Serialize, Deserialize, Clone)]
pub struct Key {
    pub verkey: String,
    pub signkey: String,
}

impl fmt::Debug for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut repr = f.debug_struct("Key");
        repr.field("verkey", &self.verkey);
        #[cfg(test)]
        repr.field("signkey", &self.signkey);
        repr.finish()
    }
}

impl Key {
    pub fn new(verkey: String, signkey: String) -> Key {
        Key {
            verkey,
            signkey,
        }
    }
}

impl Zeroize for Key {
    fn zeroize(&mut self) {
        self.signkey.zeroize();
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        self.signkey.zeroize();
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyInfo {
    pub seed: Option<String>,
    pub crypto_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeyMetadata {
    pub value: String
}
