mod helpers;
mod world;

use std::collections::HashMap;

use world::bindings::exports::wasi::http::incoming_handler::Guest;
use world::bindings::wasi::http::types::IncomingRequest;
use world::bindings::wasi::http::types::ResponseOutparam;
use world::bindings::Component;

const SENDGRID_ENDPOINT: &str = "https://api.sendgrid.com/v3/mail/send";
const DEFAULT_SUBJECT: &str = "Contact request";

impl Guest for Component {
    fn handle(req: IncomingRequest, resp: ResponseOutparam) {
        // check if settings are valid
        let settings = match Settings::from_req(&req) {
            Ok(settings) => settings,
            Err(_) => {
                let response = helpers::build_response_json_error(
                    "Failed to parse component settings, missing SendGrid API key",
                    500,
                );
                response.send(resp);
                return;
            }
        };

        // read request body
        let request_body = match helpers::parse_body(req) {
            Ok(body) => body,
            Err(e) => {
                let response = helpers::build_response_json_error(&e, 400);
                response.send(resp);
                return;
            }
        };

        // parse body to JSON
        let body_json: serde_json::Value = match serde_json::from_slice(&request_body) {
            Ok(json) => json,
            Err(_) => {
                let response =
                    helpers::build_response_json_error("Invalid JSON in request body", 400);
                response.send(resp);
                return;
            }
        };

        // extract message from request body
        let message = match body_json.get("message") {
            Some(value) => value.as_str().unwrap_or("").to_string(), // this removes quotes and converts to String
            None => {
                let response = helpers::build_response_json_error(
                    "Missing 'message' field in request body",
                    400,
                );
                response.send(resp);
                return;
            }
        };

        // extract email from request body
        let email_to = match body_json.get("email") {
            Some(value) => value.as_str().unwrap_or("").to_string(), // this removes quotes and converts to String
            None => {
                let response = helpers::build_response_json_error(
                    "Missing 'email' field in request body",
                    400,
                );
                response.send(resp);
                return;
            }
        };

        // build Slack API payload for simple text message
        let sendgrid_payload = SendGridPayload {
            personalizations: vec![SendGridPayloadPersonalizations {
                to: vec![SendGridPayloadEmail {
                    email: email_to.clone(),
                }],
                subject: settings.subject.clone(),
            }],
            from: SendGridPayloadEmail {
                email: settings.from_email.clone(),
            },
            content: vec![SendGridPayloadContent {
                _type: "text/plain".to_string(),
                value: message,
            }],
        };

        // send message to Slack
        let sendgrid_response = waki::Client::new()
            .post(SENDGRID_ENDPOINT)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", settings.api_key))
            .body(serde_json::to_vec(&sendgrid_payload).unwrap())
            .send()
            .unwrap();

        let response_status = sendgrid_response.status_code();
        let response_body =
            String::from_utf8_lossy(&sendgrid_response.body().unwrap_or_default()).to_string();

        let response = helpers::build_response_json(&response_body, response_status);
        response.send(resp);
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SendGridPayloadEmail {
    email: String
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SendGridPayload {
    personalizations: Vec<SendGridPayloadPersonalizations>,
    from: SendGridPayloadEmail,
    content: Vec<SendGridPayloadContent>,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SendGridPayloadPersonalizations {
    to: Vec<SendGridPayloadEmail>,
    subject: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SendGridPayloadContent {
    #[serde(rename = "type")]
    _type: String,
    value: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Settings {
    pub api_key: String,
    pub from_email: String,
    pub subject: String,
}

impl Settings {
    pub fn from_req(req: &IncomingRequest) -> anyhow::Result<Self> {
        let map = helpers::parse_headers(&IncomingRequest::headers(req));
        Self::new(&map)
    }

    pub fn new(headers: &HashMap<String, Vec<String>>) -> anyhow::Result<Self> {
        let settings = headers
            .get("x-edgee-component-settings")
            .ok_or_else(|| anyhow::anyhow!("Missing 'x-edgee-component-settings' header"))?;

        if settings.len() != 1 {
            return Err(anyhow::anyhow!(
                "Expected exactly one 'x-edgee-component-settings' header, found {}",
                settings.len()
            ));
        }
        let setting = settings[0].clone();
        let setting: HashMap<String, String> = serde_json::from_str(&setting)?;

        let api_key = setting
            .get("api_key")
            .map(String::to_string)
            .unwrap_or_default();
    
        let from_email = setting
            .get("from_email")
            .map(String::to_string)
            .unwrap_or_default();

        let subject = setting
            .get("subject")
            .map(String::to_string)
            .unwrap_or(DEFAULT_SUBJECT.to_string());


        Ok(Self { api_key, from_email, subject })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_new() {
        let mut headers = HashMap::new();
        headers.insert(
            "x-edgee-component-settings".to_string(),
            vec![r#"{"api_key": "test_value"}"#.to_string()],
        );

        let settings = Settings::new(&headers).unwrap();
        assert_eq!(settings.api_key, "test_value");
    }
}
