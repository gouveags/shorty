use async_trait::async_trait;

#[async_trait]
pub trait CodeGenerator: Send + Sync {
    async fn generate(&self) -> String;
}
