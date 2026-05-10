use serde_json::{json, Value};

use crate::ai::build_document_context_blocks;
use crate::types::{DocumentContextItem, TaskGenerationTurn};

/// Builds system prompt and Anthropic `messages` (user/assistant turns) for task generation.
/// `feedback_history` entries are prior rounds: each has the assistant's prior JSON list and the user's follow-up feedback.
pub fn build_generate_tasks_messages(
    project_name: &str,
    project_requirements: Option<&str>,
    feature_title: &str,
    feature_requirements: &str,
    feedback_history: &[TaskGenerationTurn],
    document_context_items: &[DocumentContextItem],
) -> (String, Vec<(String, Value)>) {
    let system = "You break down software features into concrete engineering tasks. \
Return only a JSON array (no markdown fences, no commentary). Each element must be an object with \
exactly two string fields: \"title\" and \"description\". Titles must be short and actionable; \
descriptions should clarify scope and acceptance in plain text. Propose enough tasks to cover the \
feature requirements without duplicating work."
        .to_string();

    let parent_block = project_requirements
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|ctx| {
            format!(
                "Parent project requirements (context only; tasks must stay within the feature scope):\n\
{ctx}\n\n",
            )
        })
        .unwrap_or_default();

    let initial_user_text = format!(
        "Project name:\n{project_name}\n\n\
{parent_block}\
Feature title:\n{feature_title}\n\n\
Feature requirements:\n{feature_reqs}\n\n\
Propose tasks as a JSON array only. Example shape: \
[{{\"title\":\"...\",\"description\":\"...\"}},{{\"title\":\"...\",\"description\":\"...\"}}]",
        project_name = project_name.trim(),
        parent_block = parent_block,
        feature_title = feature_title.trim(),
        feature_reqs = feature_requirements.trim(),
    );

    let mut initial_content: Vec<Value> = vec![json!({
        "type": "text",
        "text": initial_user_text,
    })];
    initial_content.extend(build_document_context_blocks(document_context_items));

    let mut messages: Vec<(String, Value)> = Vec::new();
    messages.push(("user".to_string(), Value::Array(initial_content)));

    for turn in feedback_history {
        let assistant_payload = json!(turn.proposed_tasks);
        let assistant_text = assistant_payload.to_string();
        messages.push((
            "assistant".to_string(),
            Value::String(assistant_text),
        ));
        messages.push((
            "user".to_string(),
            Value::String(turn.feedback.trim().to_string()),
        ));
    }

    (system, messages)
}
