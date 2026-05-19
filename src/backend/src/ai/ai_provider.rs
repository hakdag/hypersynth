use async_trait::async_trait;

use crate::ai::AiError;
use crate::types::{DocumentContextItem, GeneratedTaskCandidate, ProviderId, TaskGenerationTurn};

#[async_trait]
pub trait AiProvider: Send + Sync {
    fn id(&self) -> ProviderId;

    async fn list_models(&self, api_key: &str) -> Result<Vec<String>, AiError>;

    async fn enhance_requirements(
        &self,
        api_key: &str,
        project_name: &str,
        requirements: &str,
        documents: &[DocumentContextItem],
    ) -> Result<String, AiError>;

    async fn enhance_feature_requirements(
        &self,
        api_key: &str,
        project_name: &str,
        project_requirements: Option<&str>,
        feature_title: &str,
        feature_requirements: &str,
        documents: &[DocumentContextItem],
    ) -> Result<String, AiError>;

    async fn generate_tasks(
        &self,
        api_key: &str,
        project_name: &str,
        project_requirements: Option<&str>,
        feature_title: &str,
        feature_requirements: &str,
        feedback_history: &[TaskGenerationTurn],
        document_context_items: &[DocumentContextItem],
    ) -> Result<Vec<GeneratedTaskCandidate>, AiError>;
}
