use async_trait::async_trait;
use axum::http::StatusCode;
use reqwest::Client;
use serde_json::{json, Value};

use crate::ai::build_feature_requirements_system_prompt;
use crate::ai::build_feature_requirements_user_content;
use crate::ai::build_generate_tasks_messages;
use crate::ai::build_project_enhancement_system_prompt;
use crate::ai::build_project_enhancement_user_content;
use crate::ai::{AiError, AiProvider};
use crate::types::{
    AiCompletion, AiTokenUsage, DocumentContextItem, GeneratedTaskCandidate, ProviderId,
    TaskGenerationTurn,
};

pub struct AnthropicProvider {
    http: Client,
    base_url: String,
    max_tokens: u32,
}

impl AnthropicProvider {
    pub fn new(http: Client, base_url: String, _model: String, max_tokens: u32) -> Self {
        Self {
            http,
            base_url,
            max_tokens,
        }
    }

    async fn complete_enhancement(
        &self,
        api_key: &str,
        selected_model: &str,
        system: &str,
        user_content: Vec<Value>,
    ) -> Result<AiCompletion<String>, AiError> {
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let body = json!({
            "model": selected_model,
            "max_tokens": self.max_tokens,
            "system": system,
            "messages": [
                {
                    "role": "user",
                    "content": user_content
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
        extract_assistant_completion(&payload)
    }

    async fn fetch_model_ids(&self, api_key: &str) -> Result<Vec<String>, AiError> {
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

        let model_ids: Vec<String> = models
            .iter()
            .filter_map(|model| model.get("id").and_then(|id| id.as_str()))
            .map(str::to_owned)
            .collect();

        if model_ids.is_empty() {
            return Err(AiError::Empty);
        }

        Ok(model_ids)
    }
}

#[async_trait]
impl AiProvider for AnthropicProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Anthropic
    }

    async fn list_models(&self, api_key: &str) -> Result<Vec<String>, AiError> {
        self.fetch_model_ids(api_key).await
    }

    async fn enhance_requirements(
        &self,
        api_key: &str,
        selected_model: &str,
        project_name: &str,
        requirements: &str,
        documents: &[DocumentContextItem],
    ) -> Result<AiCompletion<String>, AiError> {
        let system = build_project_enhancement_system_prompt();
        let user = build_project_enhancement_user_content(project_name, requirements, documents);
        self.complete_enhancement(api_key, selected_model, &system, user)
            .await
    }

    async fn enhance_feature_requirements(
        &self,
        api_key: &str,
        selected_model: &str,
        project_name: &str,
        project_requirements: Option<&str>,
        feature_title: &str,
        feature_requirements: &str,
        documents: &[DocumentContextItem],
    ) -> Result<AiCompletion<String>, AiError> {
        let system = build_feature_requirements_system_prompt();
        let user = build_feature_requirements_user_content(
            project_name,
            project_requirements,
            feature_title,
            feature_requirements,
            documents,
        );
        self.complete_enhancement(api_key, selected_model, &system, user)
            .await
    }

    async fn generate_tasks(
        &self,
        api_key: &str,
        selected_model: &str,
        project_name: &str,
        project_requirements: Option<&str>,
        feature_title: &str,
        feature_requirements: &str,
        feedback_history: &[TaskGenerationTurn],
        document_context_items: &[DocumentContextItem],
    ) -> Result<AiCompletion<Vec<GeneratedTaskCandidate>>, AiError> {
        let (system, message_pairs) = build_generate_tasks_messages(
            project_name,
            project_requirements,
            feature_title,
            feature_requirements,
            feedback_history,
            document_context_items,
        );
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let mut messages: Vec<Value> = Vec::with_capacity(message_pairs.len());
        for (role, content) in message_pairs {
            let msg = match content {
                Value::String(s) => json!({ "role": role, "content": s }),
                Value::Array(arr) => json!({ "role": role, "content": arr }),
                _ => return Err(AiError::Decode),
            };
            messages.push(msg);
        }
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
        let completion = extract_assistant_completion(&payload)?;
        let tasks = parse_generated_tasks(&completion.value)?;
        Ok(AiCompletion {
            value: tasks,
            usage: completion.usage,
        })
    }
}

fn extract_assistant_completion(payload: &Value) -> Result<AiCompletion<String>, AiError> {
    let content = payload
        .get("content")
        .and_then(|c| c.as_array())
        .ok_or(AiError::Empty)?;
    let mut parts: Vec<&str> = Vec::new();
    for block in content {
        if let Some(t) = block.get("text").and_then(|t| t.as_str()).map(str::trim) {
            if !t.is_empty() {
                parts.push(t);
            }
        }
    }
    if parts.is_empty() {
        return Err(AiError::Empty);
    }
    let joined = parts.join("\n");
    let trimmed = joined.trim().to_string();
    if trimmed.is_empty() {
        return Err(AiError::Empty);
    }
    Ok(AiCompletion {
        value: trimmed,
        usage: parse_anthropic_usage(payload),
    })
}

fn parse_anthropic_usage(payload: &Value) -> AiTokenUsage {
    let usage = payload.get("usage");
    let input_tokens = usage
        .and_then(|u| u.get("input_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let output_tokens = usage
        .and_then(|u| u.get("output_tokens"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    AiTokenUsage {
        input_tokens: u32::try_from(input_tokens).unwrap_or(u32::MAX),
        output_tokens: u32::try_from(output_tokens).unwrap_or(u32::MAX),
    }
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
