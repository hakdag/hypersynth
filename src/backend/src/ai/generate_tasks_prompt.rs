use serde_json::json;

use crate::types::TaskGenerationTurn;

/// Builds system prompt and Anthropic `messages` (user/assistant turns) for task generation.
/// `feedback_history` entries are prior rounds: each has the assistant's prior JSON list and the user's follow-up feedback.
pub fn build_generate_tasks_messages(
    project_name: &str,
    project_requirements: Option<&str>,
    feature_title: &str,
    feature_requirements: &str,
    feedback_history: &[TaskGenerationTurn],
) -> (String, Vec<(String, String)>) {
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
                ctx = ctx
            )
        })
        .unwrap_or_default();

    let initial_user = format!(
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

    let mut messages: Vec<(String, String)> = Vec::new();
    messages.push(("user".to_string(), initial_user));

    for turn in feedback_history {
        let assistant_payload = json!(turn.proposed_tasks);
        let assistant_text = assistant_payload.to_string();
        messages.push(("assistant".to_string(), assistant_text));
        messages.push(("user".to_string(), turn.feedback.trim().to_string()));
    }

    (system, messages)
}
