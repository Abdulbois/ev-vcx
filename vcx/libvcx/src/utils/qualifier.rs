use regex::Regex;

lazy_static! {
    pub static ref REGEX: Regex = Regex::new("^(did|schema|creddef)(:?:)?(indy|cheqd)?:([a-z0-9]+):(.*)$").unwrap();
}

pub fn qualify(prefix: &str, method: &str, entity: &str) -> String {
    format!("{}:{}:{}", prefix, method, entity)
}

pub fn is_fully_qualified(entity: &str) -> bool {
    REGEX.is_match(&entity)
}

pub fn network(entity: &str) -> Option<String> {
    match REGEX.captures(entity) {
        None => None,
        Some(caps) => {
            caps.get(1).map(|m| m.as_str().to_string())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_fully_qualified_works() {
        assert!(is_fully_qualified("did:indy:some"));
        assert!(!is_fully_qualified("did:indy"));
        assert!(!is_fully_qualified("indy:some"));
    }
}