const SENDGRID_ENDPOINT: &str = "https://api.sendgrid.com/v3/mail/send";

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
struct SendGridPayloadEmail {
    email: String,
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

impl SendGridPayload {
    pub fn new(
        email_from: String,
        email_to: String,
        subject: String,
        message: Option<String>,
        template_id: Option<String>,
        dynamic_template_data: Option<serde_json::Value>,
    ) -> Self {
        if template_id.is_some() && !template_id.as_ref().unwrap().is_empty() {
            // use template if provided
            Self {
                personalizations: vec![SendGridPayloadPersonalizations {
                    to: vec![SendGridPayloadEmail { email: email_to }],
                    subject: None, // subject is not used if template_id is provided
                    dynamic_template_data,
                }],
                from: SendGridPayloadEmail { email: email_from },
                content: vec![], // no content if template_id is provided
                template_id,
            }
        } else {
            // simple text message without template
            Self {
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

    pub fn send(&self, api_key: &str) -> anyhow::Result<waki::Response> {
        let client = waki::Client::new();
        let response = client
            .post(SENDGRID_ENDPOINT)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .body(serde_json::to_vec(self)?)
            .send()?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_build_sendgrid_payload_with_template() {
        let email_from = "from@example.com".to_string();
        let email_to = "to@example.com".to_string();
        let subject = "Ignored Subject".to_string();
        let message = None;
        let template_id = Some("template-123".to_string());
        let dynamic_template_data = Some(json!({"name": "John"}));

        let payload = SendGridPayload::new(
            email_from.clone(),
            email_to.clone(),
            subject,
            message,
            template_id.clone(),
            dynamic_template_data.clone(),
        );

        assert_eq!(payload.from.email, email_from);
        assert_eq!(payload.personalizations.len(), 1);
        assert_eq!(payload.personalizations[0].to[0].email, email_to);
        assert!(payload.personalizations[0].subject.is_none());
        assert_eq!(
            payload.personalizations[0].dynamic_template_data,
            dynamic_template_data
        );
        assert_eq!(payload.content.len(), 0);
        assert_eq!(payload.template_id, template_id);
        assert_eq!(
            payload.personalizations[0].dynamic_template_data,
            dynamic_template_data
        );
    }

    #[test]
    fn test_build_sendgrid_payload_with_static_content() {
        let email_from = "from@example.com".to_string();
        let email_to = "to@example.com".to_string();
        let subject = "Hello".to_string();
        let message = Some("This is a test message.".to_string());
        let template_id = None;
        let dynamic_template_data = None;

        let payload = SendGridPayload::new(
            email_from.clone(),
            email_to.clone(),
            subject.clone(),
            message.clone(),
            template_id,
            dynamic_template_data,
        );

        assert_eq!(payload.from.email, email_from);
        assert_eq!(payload.personalizations.len(), 1);
        assert_eq!(payload.personalizations[0].to[0].email, email_to);
        assert_eq!(payload.personalizations[0].subject, Some(subject));
        assert!(payload.personalizations[0].dynamic_template_data.is_none());
        assert_eq!(payload.content.len(), 1);
        assert_eq!(payload.content[0]._type, "text/plain");
        assert_eq!(payload.content[0].value, message.unwrap());
        assert!(payload.template_id.is_none());
    }
}
