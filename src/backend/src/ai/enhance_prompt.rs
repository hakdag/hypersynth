pub fn build_prompt(project_name: &str, requirements: &str) -> (String, String) {
    let system = "You improve software project requirements for clarity and completeness. \
Return only enhanced requirements text with no preamble, no markdown code fences, and no \
explanation outside the requirements."
        .to_string();

    let user = format!(
        "Project name:\n{}\n\nCurrent project requirements:\n{}\n\nRewrite and enhance these \
requirements while preserving intent. Keep output concise, structured, and suitable for a \
rich-text editor.",
        project_name.trim(),
        requirements.trim()
    );

    (system, user)
}
