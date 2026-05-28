/// Non-data audit events: things that happened in the system but did not
/// produce a row mutation on any business table (and therefore are not
/// captured by the row-change triggers).
///
/// New variants are added as new instrumented call sites appear. The
/// string form is what gets persisted into `audit_events.event_type`.
#[derive(Debug, Clone, Copy)]
pub enum AuditEventType {
    SystemAdminLoginSuccess,
    SystemAdminLoginFailure,
    AiEnhanceProjectRequirementsRequested,
    AiEnhanceFeatureRequirementsRequested,
    AiGenerateTasksRequested,
    GlobalConfigurationChanged,
    TaskAssigneeChanged,
    TaskPriorityChanged,
    TaskDueDateChanged,
}

impl AuditEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            AuditEventType::SystemAdminLoginSuccess => "system_admin_login_success",
            AuditEventType::SystemAdminLoginFailure => "system_admin_login_failure",
            AuditEventType::AiEnhanceProjectRequirementsRequested => {
                "ai_enhance_project_requirements_requested"
            }
            AuditEventType::AiEnhanceFeatureRequirementsRequested => {
                "ai_enhance_feature_requirements_requested"
            }
            AuditEventType::AiGenerateTasksRequested => "ai_generate_tasks_requested",
            AuditEventType::GlobalConfigurationChanged => "global_configuration_changed",
            AuditEventType::TaskAssigneeChanged => "task_assignee_changed",
            AuditEventType::TaskPriorityChanged => "task_priority_changed",
            AuditEventType::TaskDueDateChanged => "task_due_date_changed",
        }
    }
}
