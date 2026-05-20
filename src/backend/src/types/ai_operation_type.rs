#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiOperationType {
    EnhanceProjectRequirements,
    #[allow(dead_code)]
    SplitProjectIntoFeatures,
    EnhanceFeatureRequirements,
    GenerateTasks,
    RegenerateTasks,
}

impl AiOperationType {
    pub fn as_str(self) -> &'static str {
        match self {
            AiOperationType::EnhanceProjectRequirements => "enhance_project_requirements",
            AiOperationType::SplitProjectIntoFeatures => "split_project_into_features",
            AiOperationType::EnhanceFeatureRequirements => "enhance_feature_requirements",
            AiOperationType::GenerateTasks => "generate_tasks",
            AiOperationType::RegenerateTasks => "regenerate_tasks",
        }
    }
}
