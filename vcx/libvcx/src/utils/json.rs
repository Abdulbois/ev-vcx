use serde_json::Value;

/// Renames keys in a JSON map based on a provided callback -- unmatched keys are left untouched
pub fn mapped_key_rewrite(val: Value, remap: fn(&str, Option<&str>) -> Option<String>) -> Value {
    _mapped_key_rewrite(val, None, remap)
}

fn _mapped_key_rewrite(
    val: Value,
    parent: Option<&str>,
    remap: fn(&str, Option<&str>) -> Option<String>,
) -> Value {
    if let Value::Object(map) = val {
        Value::Object(
            map.into_iter()
                .map(|(k, v)| {
                    let new_k = remap(&k, parent).unwrap_or(k);
                    let new_v = _mapped_key_rewrite(v, Some(&new_k), remap);
                    (new_k, new_v)
                })
                .collect(),
        )
    } else {
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn simple() {
        let remap = |key: &str, _parent: Option<&str>| {
            let new_key = match key {
                "d" => "devin",
                _ => return None,
            };

            Some(new_key.to_string())
        };
        let simple = json!({"d":"d"});
        let expected = json!({"devin":"d"});
        let transformed = mapped_key_rewrite(simple, remap);
        assert_eq!(expected, transformed);

        let simple = json!(null);
        let transformed = mapped_key_rewrite(simple.clone(), remap);
        assert_eq!(simple, transformed);

        let simple = json!("null");
        let transformed = mapped_key_rewrite(simple.clone(), remap);
        assert_eq!(simple, transformed);
    }

    #[test]
    fn abbr_test() {
        let un_abbr = json!({
            "statusCode":"MS-102",
            "connReqId":"yta2odh",
            "senderDetail":{
                "name":"ent-name",
                "agentKeyDlgProof":{
                    "agentDID":"N2Uyi6SVsHZq1VWXuA3EMg",
                    "agentDelegatedKey":"CTfF2sZ5q4oPcBvTP75pgx3WGzYiLSTwHGg9zUsJJegi",
                    "signature":"/FxHMzX8JaH461k1SI5PfyxF5KwBAe6VlaYBNLI2aSZU3APsiWBfvSC+mxBYJ/zAhX9IUeTEX67fj+FCXZZ2Cg=="
                },
                "DID":"F2axeahCaZfbUYUcKefc3j",
                "logoUrl":"ent-logo-url",
                "verKey":"74xeXSEac5QTWzQmh84JqzjuXc8yvXLzWKeiqyUnYokx"
            },
            "senderAgencyDetail":{
                "DID":"BDSmVkzxRYGE4HKyMKxd1H",
                "verKey":"6yUatReYWNSUfEtC2ABgRXmmLaxCyQqsjLwv2BomxsxD",
                "endpoint":"52.38.32.107:80/agency/msg"
            },
            "targetName":"there",
            "statusMsg":"message sent"
        });

        let abbr = json!({
            "sc":"MS-102",
            "id": "yta2odh",
            "s": {
                "n": "ent-name",
                "dp": {
                    "d": "N2Uyi6SVsHZq1VWXuA3EMg",
                    "k": "CTfF2sZ5q4oPcBvTP75pgx3WGzYiLSTwHGg9zUsJJegi",
                    "s": "/FxHMzX8JaH461k1SI5PfyxF5KwBAe6VlaYBNLI2aSZU3APsiWBfvSC+mxBYJ/zAhX9IUeTEX67fj+FCXZZ2Cg==",
                },
                "d": "F2axeahCaZfbUYUcKefc3j",
                "l": "ent-logo-url",
                "v": "74xeXSEac5QTWzQmh84JqzjuXc8yvXLzWKeiqyUnYokx",
            },
            "sa": {
                "d": "BDSmVkzxRYGE4HKyMKxd1H",
                "v": "6yUatReYWNSUfEtC2ABgRXmmLaxCyQqsjLwv2BomxsxD",
                "e": "52.38.32.107:80/agency/msg",
            },
            "t": "there",
            "sm":"message sent"
        });
        let transformed = mapped_key_rewrite(un_abbr, |key: &str, _parent: Option<&str>| {
            let new_key = match key {
                "statusCode" => "sc",
                "connReqId" => "id",
                "senderDetail" => "s",
                "name" => "n",
                "agentKeyDlgProof" => "dp",
                "agentDID" => "d",
                "agentDelegatedKey" => "k",
                "signature" => "s",
                "DID" => "d",
                "logoUrl" => "l",
                "verKey" => "v",
                "senderAgencyDetail" => "sa",
                "endpoint" => "e",
                "targetName" => "t",
                "statusMsg" => "sm",
                _ => return None,
            };

            Some(new_key.to_string())
        });

        assert_eq!(abbr, transformed);
    }
}

