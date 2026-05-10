use serde_json::{json, Value};

use crate::ai::build_document_context_blocks;
use crate::types::DocumentContextItem;

pub fn build_project_enhancement_system_prompt() -> String {
    "You improve software project requirements for clarity and completeness. \
The target format is plain Markdown as in a single .md file (headings, lists, emphasis as needed). \
Return only that Markdown body with no preamble, no wrapping markdown code fences around the \
whole document, and no explanation outside the requirements."
        .to_string()
}

pub fn build_project_enhancement_user_content(
    project_name: &str,
    requirements: &str,
    documents: &[DocumentContextItem],
) -> Vec<Value> {
    let text = format!(
        "Project name:\n{}\n\nCurrent project requirements:\n{}\n\nRewrite and enhance these \
requirements while preserving intent. Keep output concise, structured, and valid Markdown \
suitable for saving as a .md file.",
        project_name.trim(),
        requirements.trim(),
    );

    let mut blocks: Vec<Value> = Vec::with_capacity(1 + documents.len());
    blocks.push(json!({
        "type": "text",
        "text": text,
    }));
    blocks.extend(build_document_context_blocks(documents));

    blocks
}

/// Feature enhancement system prompt text.
pub fn build_feature_requirements_system_prompt() -> String {
    "You improve software feature requirements for clarity and completeness within a project. \
The target format is plain Markdown as in a single .md file (headings, lists, emphasis as needed). \
Return only that Markdown body with no preamble, no wrapping markdown code fences around the \
whole document, and no explanation outside the requirements."
        .to_string()
}

pub fn build_feature_requirements_user_content(
    project_name: &str,
    project_requirements: Option<&str>,
    feature_title: &str,
    feature_requirements: &str,
    documents: &[DocumentContextItem],
) -> Vec<Value> {
    let parent_block = project_requirements
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|ctx| {
            format!(
                "Parent project requirements (context only; do not replace the feature scope with the whole project):\n\
{ctx}\n\n",
            )
        })
        .unwrap_or_default();

    let text = format!(
        "Project name:\n{project_name}\n\n\
{parent_block}\
Feature title:\n{feature_title}\n\n\
Current feature requirements:\n{feature_reqs}\n\n\
Rewrite and enhance these feature requirements while preserving intent. Align with the parent project \
context when it is provided. Keep output concise, structured, and valid Markdown suitable for saving as a .md file.",
        project_name = project_name.trim(),
        parent_block = parent_block,
        feature_title = feature_title.trim(),
        feature_reqs = feature_requirements.trim(),
    );

    let mut blocks: Vec<Value> = Vec::with_capacity(1 + documents.len());
    blocks.push(json!({
        "type": "text",
        "text": text,
    }));
    blocks.extend(build_document_context_blocks(documents));

    blocks
}
