use std::collections::HashMap;
use std::io::Read;

use crate::error::prelude::*;
use crate::v3::messages::connection::did_doc::Service;
use reqwest::{Url, Client, ClientBuilder, RedirectPolicy};
use reqwest::Response;

pub mod encryption_envelope;

// ensure service keys are naked keys
pub(crate) fn normalize_service_keys(services: &mut Vec<Service>) -> VcxResult<()> {
    for service in services.iter_mut() {
        Service::transform_did_keys_to_naked_keys(&mut service.recipient_keys)?;
        if !service.routing_keys.is_empty() {
            Service::transform_did_keys_to_naked_keys(&mut service.routing_keys)?
        }
    }

    Ok(())
}

pub(crate) fn resolve_message_by_url(url: &str) -> VcxResult<String> {
    let parsed_url: Url = Url::parse(url)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::InvalidUrl,
            format!("Unable to parse URL from the given string. Err: {:?}", err),
        ))?;

    let query_params: HashMap<String, String> = parsed_url.query_pairs().into_owned().collect();

    // 1. Check if the message inside of query parameters as base64 encoded string
    let embedded_message =
        query_params.get("c_i")
            .or(query_params.get("d_m"))
            .or(query_params.get("m"))
            .or(query_params.get("oob"));

    if let Some(embedded_message) = embedded_message {
        if let Ok(message) = decode_query_message(&embedded_message, base64::STANDARD) {
            return Ok(message);
        }

        if let Ok(message) = decode_query_message(&embedded_message, base64::STANDARD_NO_PAD) {
            return Ok(message);
        }

        if let Ok(message) = decode_query_message(&embedded_message, base64::URL_SAFE) {
            return Ok(message);
        }

        if let Ok(message) = decode_query_message(&embedded_message, base64::URL_SAFE_NO_PAD) {
            return Ok(message);
        }
    }

    // Return. Do not query for url starting from didcomm:
    if url.starts_with("didcomm:") {
        return Err(VcxError::from_msg(
            VcxErrorKind::InvalidUrl,
            "Unable to get message from the given URL",
        ));
    }

    // 2. Send GET request
    let client: Client =
        ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(50))
            .redirect(RedirectPolicy::none())
            .build()
            .map_err(|err| VcxError::from_msg(
                VcxErrorKind::InvalidUrl
                , format!("Unable to query message for the given URL. Err: {:?}", err),
            ))?;

    let mut response: Response =
        client.get(url)
            .send()
            .map_err(|err| VcxError::from_msg(
                VcxErrorKind::InvalidUrl,
                format!("Unable to query message for the given URL. Err: {:?}", err),
            ))?;

    // 3. Request returned REDIRECTION with `location` header
    if response.status().is_redirection() {
        let location = response.headers().get("location")
            .ok_or(VcxError::from_msg(
                VcxErrorKind::InvalidUrl,
                "Unable to get message from the given URL",
            ))?;
        let location = location.to_str()
            .map_err(|_| VcxError::from_msg(
                VcxErrorKind::InvalidUrl,
                "Unable to get message from the given URL",
            ))?;


        return resolve_message_by_url(location);
    }

    // 4. Request returned OK with value
    let mut content = String::new();
    if response.status().is_success() {
        response.read_to_string(&mut content)
            .map_err(|err| VcxError::from_msg(
                VcxErrorKind::InvalidUrl,
                format!("Unable to query message for the given URL. Err: {:?}", err),
            ))?;
    }

    // 5. Request result is URL - handle it again
    if let Ok(_) = Url::parse(&content) {
        return resolve_message_by_url(&content);
    }

    serde_json::from_str::<serde_json::Value>(&content)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::InvalidUrl,
            format!("Unable to parse JSON object from the response. Err: {:?}", err),
        ))?;

    Ok(content)
}

fn decode_query_message(message: &str, config: base64::Config) -> VcxResult<String> {
    let message = base64::decode_config(message, config)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::InvalidUrl,
            format!("Unable to decode base64 message. Err: {:?}", err)
        ))?;

    let message = std::str::from_utf8(&message)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::InvalidUrl,
            format!("Unable to parse message from the given bytes. Err: {:?}", err)
        ))?;

    serde_json::from_str::<serde_json::Value>(&message)
        .map_err(|err| VcxError::from_msg(
            VcxErrorKind::InvalidUrl,
            format!("Unable to parse JSON object from the response. Err: {:?}", err),
        ))?;

    Ok(message.to_string())
}

#[cfg(test)]
pub mod tests {
    use super::*;

    const MESSAGE: &str = r#"{"value":"ok"}"#;

    #[test]
    fn test_resolve_message_by_url_containing_base64_encoded_query_parameter() {
        // with padding
        let url = "https://vas.evernym.com/agency/msg?oob=eyJ2YWx1ZSI6Im9rIn0=";
        assert_eq!(MESSAGE, resolve_message_by_url(url).unwrap());

        // without padding
        let url = "didcomm://?c_i=eyJ2YWx1ZSI6Im9rIn0";
        assert_eq!(MESSAGE, resolve_message_by_url(url).unwrap());
    }
}
