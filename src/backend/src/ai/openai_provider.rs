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
use crate::types::{DocumentContextItem, GeneratedTaskCandidate, ProviderId, TaskGenerationTurn};

pub struct OpenAiProvider {
    http: Client,
    base_url: String,
    max_tokens: u32,
}

impl OpenAiProvider {
    pub fn new(http: Client, base_url: String, _default_model: String, max_tokens: u32) -> Self {
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
    ) -> Result<String, AiError> {
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );
        let body = json!({
            "model": selected_model,
            "max_tokens": self.max_tokens,
            "messages": [
                {
                    "role": "system",
                    "content": system
                },
                {
                    "role": "user",
                    "content": openai_content_from_anthropic(Value::Array(user_content))?
                }
            ]
        });

        let response = self
            .http
            .post(url)
            .bearer_auth(api_key)
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
        extract_chat_completion_text(&payload)
    }
    async fn fetch_model_ids(&self, api_key: &str) -> Result<Vec<String>, AiError> {
        let url = format!("{}/v1/models", self.base_url.trim_end_matches('/'));
        let response = self
            .http
            .get(url)
            .bearer_auth(api_key)
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
impl AiProvider for OpenAiProvider {
    fn id(&self) -> ProviderId {
        ProviderId::OpenAi
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
    ) -> Result<String, AiError> {
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
    ) -> Result<String, AiError> {
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
    ) -> Result<Vec<GeneratedTaskCandidate>, AiError> {
        let (system, message_pairs) = build_generate_tasks_messages(
            project_name,
            project_requirements,
            feature_title,
            feature_requirements,
            feedback_history,
            document_context_items,
        );
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );
        let mut messages: Vec<Value> = Vec::with_capacity(message_pairs.len() + 1);
        messages.push(json!({
            "role": "system",
            "content": system,
        }));
        for (role, content) in message_pairs {
            messages.push(json!({
                "role": role,
                "content": openai_content_from_anthropic(content)?,
            }));
        }
        let body = json!({
            "model": selected_model,
            "max_tokens": self.max_tokens,
            "messages": messages,
        });

        let response = self
            .http
            .post(url)
            .bearer_auth(api_key)
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
        let text = extract_chat_completion_text(&payload)?;
        parse_generated_tasks(&text)
    }
}

fn openai_content_from_anthropic(content: Value) -> Result<Value, AiError> {
    match content {
        Value::String(text) => Ok(Value::String(text)),
        Value::Array(blocks) => {
            let mut converted: Vec<Value> = Vec::with_capacity(blocks.len());
            for block in blocks {
                converted.push(convert_openai_content_block(block)?);
            }
            Ok(Value::Array(converted))
        }
        _ => Err(AiError::Decode),
    }
}

fn convert_openai_content_block(block: Value) -> Result<Value, AiError> {
    let Some(block_type) = block.get("type").and_then(|value| value.as_str()) else {
        return Err(AiError::Decode);
    };

    match block_type {
        "text" => {
            let Some(text) = block.get("text").and_then(|value| value.as_str()) else {
                return Err(AiError::Decode);
            };
            Ok(json!({
                "type": "text",
                "text": text,
            }))
        }
        "image" => {
            let Some(source) = block.get("source").and_then(|value| value.as_object()) else {
                return Err(AiError::Decode);
            };
            let Some(media_type) = source.get("media_type").and_then(|value| value.as_str()) else {
                return Err(AiError::Decode);
            };
            let Some(data_base64) = source.get("data").and_then(|value| value.as_str()) else {
                return Err(AiError::Decode);
            };
            Ok(json!({
                "type": "image_url",
                "image_url": {
                    "url": format!("data:{media_type};base64,{data_base64}"),
                },
            }))
        }
        _ => Err(AiError::Decode),
    }
}

fn extract_chat_completion_text(payload: &Value) -> Result<String, AiError> {
    let text = payload
        .get("choices")
        .and_then(|choices| choices.as_array())
        .and_then(|choices| choices.first())
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .map(str::trim)
        .filter(|content| !content.is_empty())
        .ok_or(AiError::Empty)?;

    Ok(text.to_string())
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
