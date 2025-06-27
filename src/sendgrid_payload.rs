#[derive(serde::Deserialize, serde::Serialize)]
struct SendGridPayloadEmail {
    email: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct SendGridPayload {
    personalizations: Vec<SendGridPayloadPersonalizations>,
    from: SendGridPayloadEmail,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    content: Vec<SendGridPayloadContent>, // used only if no template_id is provided via settings
    #[serde(skip_serializing_if = "Option::is_none")]
    template_id: Option<String>, // used only if provided via settings
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SendGridPayloadPersonalizations {
    to: Vec<SendGridPayloadEmail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>, // used if no template_id is provided
    #[serde(skip_serializing_if = "Option::is_none")]
    dynamic_template_data: Option<serde_json::Value>, // used if template_id is provided
}

#[derive(serde::Deserialize, serde::Serialize)]
struct SendGridPayloadContent {
    #[serde(rename = "type")]
    _type: String,
    value: String,
}

pub fn build_sendgrid_payload(
    email_from: String,
    email_to: String,
    subject: String,
    message: Option<String>,
    template_id: Option<String>,
    template_data: Option<serde_json::Value>,
) -> SendGridPayload {
    if template_id.is_some() {
        // use template if provided
        SendGridPayload {
            personalizations: vec![SendGridPayloadPersonalizations {
                to: vec![SendGridPayloadEmail { email: email_to }],
                subject: None, // subject is not used if template_id is provided
                dynamic_template_data: template_data,
            }],
            from: SendGridPayloadEmail { email: email_from },
            content: vec![], // no content if template_id is provided
            template_id: template_id,
        }
    } else {
        // simple text message without template
        SendGridPayload {
            personalizations: vec![SendGridPayloadPersonalizations {
                to: vec![SendGridPayloadEmail { email: email_to }],
                subject: Some(subject),
                dynamic_template_data: None,
            }],
            from: SendGridPayloadEmail { email: email_from },
            content: vec![SendGridPayloadContent {
                _type: "text/plain".to_string(),
                value: message.unwrap().to_string(),
            }],
            template_id: None, // use content if no template_id is provided
        }
    }
}
