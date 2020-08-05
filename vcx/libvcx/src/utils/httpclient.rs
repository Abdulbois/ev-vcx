use settings;
use std::io::Read;
use std::sync::Mutex;
use reqwest::header::CONTENT_TYPE;
use std::env;
use error::prelude::*;
use utils::timeout::TimeoutUtils;
use reqwest::ClientBuilder;

lazy_static! {
    static ref AGENCY_MOCK: Mutex<AgencyMock> = Mutex::new(AgencyMock::default());
}

#[derive(Default)]
pub struct AgencyMock {
    responses: Vec<Vec<u8>>
}

impl AgencyMock {
    pub fn set_next_response(body: Vec<u8>) {
        if settings::agency_mocks_enabled() {
            AGENCY_MOCK.lock().unwrap().responses.push(body);
        }
    }

    pub fn get_response() -> VcxResult<Vec<u8>> {
        Ok(AGENCY_MOCK.lock().unwrap().responses.pop().unwrap_or_default())
    }
}

//Todo: change this RC to a u32
pub fn post_u8(body_content: &Vec<u8>) -> VcxResult<Vec<u8>> {
    let endpoint = format!("{}/agency/msg", settings::get_config_value(settings::CONFIG_AGENCY_ENDPOINT)?);
    post_message(body_content, &endpoint)
}

pub fn post_message(body_content: &Vec<u8>, url: &str) -> VcxResult<Vec<u8>> {
    if settings::agency_mocks_enabled() {
        return AgencyMock::get_response();
    }

    //Setting SSL Certs location. This is needed on android platform. Or openssl will fail to verify the certs
    if cfg!(target_os = "android") {
        info!("::Android code");
        set_ssl_cert_location();
    }
    let client =
        ClientBuilder::new()
            .timeout(TimeoutUtils::long_timeout())
            .build()
            .map_err(|err| VcxError::from_msg(VcxErrorKind::PostMessageFailed, format!("Could not prepare HTTP client. Err: {:?}", err)))?;

    debug!("Posting encrypted bundle to: \"{}\"", secret!(url));

    let mut response =
        client.post(url)
            .body(body_content.to_owned())
            .header(CONTENT_TYPE, "application/ssi-agent-wire")
            .send()
            .map_err(|err| {
                error!("error: {}", err);
                VcxError::from_msg(VcxErrorKind::PostMessageFailed, format!("Could send HTTP message. Error: {:?}", err))
            })?;

    trace!("Response Header: {:?}", response);
    if !response.status().is_success() {
        let mut content = String::new();
        match response.read_to_string(&mut content) {
            Ok(_) => info!("Request failed: {}", content),
            Err(_) => info!("Could not read response"),
        };
        return Err(VcxError::from_msg(VcxErrorKind::PostMessageFailed, format!("Sending POST HTTP request failed with: {}", content)));
    }

    let mut content = Vec::new();
    response.read_to_end(&mut content)
        .map_err(|err| VcxError::from_msg(VcxErrorKind::PostMessageFailed, format!("Could not read HTTP response. Err: {:?}", err)))?;

    Ok(content)
}

fn set_ssl_cert_location() {
    let ssl_cert_file = "SSL_CERT_FILE";
    env::set_var(ssl_cert_file, env::var("EXTERNAL_STORAGE").unwrap() + "/cacert.pem"); //TODO: CHANGE ME, HARDCODING FOR TESTING ONLY
    match env::var(ssl_cert_file) {
        Ok(val) => info!("{}:: {:?}", ssl_cert_file, val),
        Err(e) => error!("couldn't find var in env {}:: {}. This needs to be set on Android to make https calls.\n See https://github.com/seanmonstar/reqwest/issues/70 for more info",
                         ssl_cert_file, e),
    }
    info!("::SSL_CERT_FILE has been set");
}
