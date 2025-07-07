mod helpers;
mod sendgrid_payload;
mod world;

use std::collections::HashMap;

use sendgrid_payload::build_sendgrid_payload;
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

        let message = match extract_message(&body_json, &settings.template_id) {
            Ok(data) => data,
            Err(e) => {
                let response = helpers::build_response_json_error(&e.to_string(), 400);
                response.send(resp);
                return;
            }
        };

        let template_data = match extract_template_data(&body_json, &settings.template_id) {
            Ok(data) => data,
            Err(e) => {
                let response = helpers::build_response_json_error(&e.to_string(), 400);
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
        let sendgrid_payload = build_sendgrid_payload(
            settings.email_from,
            email_to,
            settings.subject,
            message,
            settings.template_id,
            template_data,
        );

        // send message to SendGrid
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

fn extract_message(
    body_json: &serde_json::Value,
    template_id: &Option<String>,
) -> anyhow::Result<Option<String>> {
    match body_json.get("message") {
        // just return the value if it exists (removing quotes and converting to String)
        Some(value) => Ok(Some(value.as_str().unwrap_or("").to_string())),
        None => {
            // if template_id is provided, message is not required
            if template_id.is_some() && !template_id.as_ref().unwrap().is_empty() {
                Ok(None)
            } else {
                Err(anyhow::anyhow!("Missing 'message' field in request body"))
            }
        }
    }
}

fn extract_template_data(
    body_json: &serde_json::Value,
    template_id: &Option<String>,
) -> anyhow::Result<Option<serde_json::Value>> {
    match body_json.get("data") {
        // just return the value if it exists
        Some(value) => Ok(Some(value.clone())),
        None => {
            // if template_id is not provided, data is not required
            if template_id.is_none() || template_id.as_ref().unwrap().is_empty() {
                Ok(None)
            } else {
                Err(anyhow::anyhow!("Missing 'data' field in request body"))
            }
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Settings {
    pub api_key: String,
    pub email_from: String,
    pub subject: String,             // optional, defaults to "Contact request"
    pub template_id: Option<String>, // optional
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

        let email_from = setting
            .get("email_from")
            .map(String::to_string)
            .unwrap_or_default();

        let subject = setting
            .get("subject")
            .map(String::to_string)
            .unwrap_or(DEFAULT_SUBJECT.to_string());

        let template_id: Option<String> = setting.get("template_id").cloned();

        Ok(Self {
            api_key,
            email_from,
            subject,
            template_id,
        })
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

    #[test]
    fn test_settings_new_missing_header() {
        let headers = HashMap::new();
        let result = Settings::new(&headers);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Missing 'x-edgee-component-settings' header"
        );
    }

    #[test]
    fn test_settings_new_multiple_headers() {
        let mut headers = HashMap::new();
        headers.insert(
            "x-edgee-component-settings".to_string(),
            vec![
                r#"{"api_key": "test_value"}"#.to_string(),
                r#"{"api_key": "another_value"}"#.to_string(),
            ],
        );
        let result = Settings::new(&headers);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Expected exactly one 'x-edgee-component-settings' header"));
    }

    #[test]
    fn test_settings_new_invalid_json() {
        let mut headers = HashMap::new();
        headers.insert(
            "x-edgee-component-settings".to_string(),
            vec!["not a json".to_string()],
        );
        let result = Settings::new(&headers);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_message_with_message() {
        let json = serde_json::json!({"message": "Hello, world!"});
        let template_id = None;
        let result = extract_message(&json, &template_id).unwrap();
        assert_eq!(result, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_extract_message_with_message_empty_template_id() {
        let json = serde_json::json!({"message": "Hello, world!"});
        let template_id = Some("".to_string()); //empty string instead of None
        let result = extract_message(&json, &template_id).unwrap();
        assert_eq!(result, Some("Hello, world!".to_string()));
    }

    #[test]
    fn test_extract_message_missing_message_with_template_id() {
        let json = serde_json::json!({});
        let template_id = Some("template123".to_string());
        let result = extract_message(&json, &template_id).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_extract_message_missing_message_without_template_id() {
        let json = serde_json::json!({});
        let template_id = None;
        let result = extract_message(&json, &template_id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Missing 'message' field in request body"
        );
    }

    #[test]
    fn test_extract_message_missing_message_with_empty_template_id() {
        let json = serde_json::json!({});
        let template_id = Some("".to_string()); //empty string instead of None
        let result = extract_message(&json, &template_id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Missing 'message' field in request body"
        );
    }

    #[test]
    fn test_extract_template_data_with_data() {
        let json = serde_json::json!({"data": {"foo": "bar"}});
        let template_id = Some("template123".to_string());
        let result = extract_template_data(&json, &template_id).unwrap();
        assert_eq!(result, Some(serde_json::json!({"foo": "bar"})));
    }

    #[test]
    fn test_extract_template_data_missing_data_with_template_id() {
        let json = serde_json::json!({});
        let template_id = Some("template123".to_string());
        let result = extract_template_data(&json, &template_id);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Missing 'data' field in request body"
        );
    }

    #[test]
    fn test_extract_template_data_missing_data_without_template_id() {
        let json = serde_json::json!({});
        let template_id = None;
        let result = extract_template_data(&json, &template_id).unwrap();
        assert_eq!(result, None);
    }
}
