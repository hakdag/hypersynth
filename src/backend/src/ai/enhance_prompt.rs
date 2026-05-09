pub fn build_prompt(project_name: &str, requirements: &str) -> (String, String) {
    let system = "You improve software project requirements for clarity and completeness. \
The target format is plain Markdown as in a single .md file (headings, lists, emphasis as needed). \
Return only that Markdown body with no preamble, no wrapping markdown code fences around the \
whole document, and no explanation outside the requirements."
        .to_string();

    let user = format!(
        "Project name:\n{}\n\nCurrent project requirements:\n{}\n\nRewrite and enhance these \
requirements while preserving intent. Keep output concise, structured, and valid Markdown \
suitable for saving as a .md file.",
        project_name.trim(),
        requirements.trim()
    );

    (system, user)
}

/// Builds prompts for enhancing a single feature's requirements using optional parent project requirements.
pub fn build_feature_requirements_prompt(
    project_name: &str,
    project_requirements: Option<&str>,
    feature_title: &str,
    feature_requirements: &str,
) -> (String, String) {
    let system =
        "You improve software feature requirements for clarity and completeness within a project. \
The target format is plain Markdown as in a single .md file (headings, lists, emphasis as needed). \
Return only that Markdown body with no preamble, no wrapping markdown code fences around the \
whole document, and no explanation outside the requirements."
            .to_string();

    let parent_block = project_requirements
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|ctx| {
            format!(
                "Parent project requirements (context only; do not replace the feature scope with the whole project):\n\
{ctx}\n\n",
                ctx = ctx
            )
        })
        .unwrap_or_default();

    let user = format!(
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

    (system, user)
}
