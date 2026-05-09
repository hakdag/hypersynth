use axum::http::StatusCode;
use reqwest::Client;
use serde_json::{json, Value};

use crate::ai::build_feature_requirements_prompt;
use crate::ai::build_generate_tasks_messages;
use crate::ai::build_prompt;
use crate::ai::AiError;
use crate::types::{GeneratedTaskCandidate, TaskGenerationTurn};

#[derive(Clone)]
pub struct AnthropicClient {
    http: Client,
    base_url: String,
    model: String,
    max_tokens: u32,
}

impl AnthropicClient {
    pub fn new(http: Client, base_url: String, model: String, max_tokens: u32) -> Self {
        Self {
            http,
            base_url,
            model,
            max_tokens,
        }
    }

    pub async fn enhance_requirements(
        &self,
        api_key: &str,
        project_name: &str,
        requirements: &str,
    ) -> Result<String, AiError> {
        let (system, user) = build_prompt(project_name, requirements);
        self.complete_enhancement(api_key, &system, &user).await
    }

    pub async fn enhance_feature_requirements(
        &self,
        api_key: &str,
        project_name: &str,
        project_requirements: Option<&str>,
        feature_title: &str,
        feature_requirements: &str,
    ) -> Result<String, AiError> {
        let (system, user) = build_feature_requirements_prompt(
            project_name,
            project_requirements,
            feature_title,
            feature_requirements,
        );
        self.complete_enhancement(api_key, &system, &user).await
    }

    pub async fn generate_tasks(
        &self,
        api_key: &str,
        project_name: &str,
        project_requirements: Option<&str>,
        feature_title: &str,
        feature_requirements: &str,
        feedback_history: &[TaskGenerationTurn],
    ) -> Result<Vec<GeneratedTaskCandidate>, AiError> {
        let (system, message_pairs) = build_generate_tasks_messages(
            project_name,
            project_requirements,
            feature_title,
            feature_requirements,
            feedback_history,
        );
        let selected_model = self.resolve_haiku_model(api_key).await?;
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let messages: Vec<Value> = message_pairs
            .into_iter()
            .map(|(role, content)| json!({ "role": role, "content": content }))
            .collect();
        let body = json!({
            "model": selected_model,
            "max_tokens": self.max_tokens,
            "system": system,
            "messages": messages
        });

        let response = self
            .http
            .post(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|_| AiError::Network)?;

        if !response.status().is_success() {
            return Err(AiError::Provider(
                StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            ));
        }

        let payload: Value = response.json().await.map_err(|_| AiError::Decode)?;
        let text = payload
            .get("content")
            .and_then(|content| content.as_array())
            .and_then(|content| content.first())
            .and_then(|first| first.get("text"))
            .and_then(|text| text.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or(AiError::Empty)?;

        parse_generated_tasks(text)
    }

    async fn complete_enhancement(
        &self,
        api_key: &str,
        system: &str,
        user: &str,
    ) -> Result<String, AiError> {
        let selected_model = self.resolve_haiku_model(api_key).await?;
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let body = json!({
            "model": selected_model,
            "max_tokens": self.max_tokens,
            "system": system,
            "messages": [
                {
                    "role": "user",
                    "content": user
                }
            ]
        });

        let response = self
            .http
            .post(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|_| AiError::Network)?;

        if !response.status().is_success() {
            return Err(AiError::Provider(
                StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            ));
        }

        let payload: Value = response.json().await.map_err(|_| AiError::Decode)?;
        let text = payload
            .get("content")
            .and_then(|content| content.as_array())
            .and_then(|content| content.first())
            .and_then(|first| first.get("text"))
            .and_then(|text| text.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or(AiError::Empty)?;

        Ok(text.to_string())
    }

    async fn resolve_haiku_model(&self, api_key: &str) -> Result<String, AiError> {
        let url = format!("{}/v1/models", self.base_url.trim_end_matches('/'));
        let response = self
            .http
            .get(url)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .send()
            .await
            .map_err(|_| AiError::Network)?;

        if !response.status().is_success() {
            return Err(AiError::Provider(
                StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY),
            ));
        }

        let payload: Value = response.json().await.map_err(|_| AiError::Decode)?;
        let Some(models) = payload.get("data").and_then(|data| data.as_array()) else {
            return Err(AiError::Decode);
        };

        let best_haiku = models
            .iter()
            .filter_map(|model| model.get("id").and_then(|id| id.as_str()))
            .filter(|id| id.starts_with("claude-haiku-"))
            .max_by_key(|id| parse_model_rank(id));

        if let Some(model_id) = best_haiku {
            return Ok(model_id.to_string());
        }

        if self.model.starts_with("claude-haiku-") {
            return Ok(self.model.clone());
        }

        Err(AiError::Empty)
    }
}

fn parse_model_rank(model_id: &str) -> (u32, u32, u32) {
    let version = model_id.strip_prefix("claude-haiku-").unwrap_or("");
    let mut parts = version.split('-');

    let major = parts
        .next()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    let minor = parts
        .next()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);
    let snapshot = parts
        .next()
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    (major, minor, snapshot)
}

fn strip_markdown_code_fence(s: &str) -> &str {
    let t = s.trim();
    let Some(rest) = t.strip_prefix("```") else {
        return t;
    };
    let body = rest
        .find('\n')
        .map(|i| &rest[i + 1..])
        .unwrap_or(rest)
        .trim_start();
    if let Some(idx) = body.rfind("```") {
        body[..idx].trim()
    } else {
        body.trim()
    }
}

fn parse_generated_tasks(raw: &str) -> Result<Vec<GeneratedTaskCandidate>, AiError> {
    let trimmed = strip_markdown_code_fence(raw);
    let arr: Vec<Value> = serde_json::from_str(trimmed).map_err(|_| AiError::Decode)?;
    if arr.is_empty() {
        return Err(AiError::Empty);
    }
    let mut out = Vec::new();
    for value in arr {
        let Some(obj) = value.as_object() else {
            return Err(AiError::Decode);
        };
        let title = obj
            .get("title")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or(AiError::Decode)?;
        let description = obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        out.push(GeneratedTaskCandidate {
            title: title.to_string(),
            description,
        });
    }
    if out.is_empty() {
        return Err(AiError::Empty);
    }
    Ok(out)
}
