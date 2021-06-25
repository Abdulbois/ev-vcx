#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ProtocolTypes {
    #[serde(rename = "1.0")]
    V1,
    #[serde(rename = "2.0")]
    V2,
    #[serde(rename = "3.0")]
    V3,
}

impl Default for ProtocolTypes {
    fn default() -> Self {
        ProtocolTypes::V1
    }
}

impl From<String> for ProtocolTypes {
    fn from(type_: String) -> Self {
        match type_.as_str() {
            "1.0" => ProtocolTypes::V1,
            "2.0" => ProtocolTypes::V2,
            "3.0" => ProtocolTypes::V3,
            type_ @ _ => {
                error!("Unknown protocol type: {:?}. Use default", type_);
                ProtocolTypes::default()
            }
        }
    }
}

impl ::std::string::ToString for ProtocolTypes {
    fn to_string(&self) -> String {
        match self {
            ProtocolTypes::V1 => "1.0".to_string(),
            ProtocolTypes::V2 => "2.0".to_string(),
            ProtocolTypes::V3 => "3.0".to_string(),
        }
    }
}