pub mod dpr;
pub mod openai;
pub mod sentence_transformers;

use crate::embeddings::dpr::DPREmbeddings;

use super::server_config::{
    self, EmbeddingModelKind::AllMiniLmL12V2, EmbeddingModelKind::AllMiniLmL6V2,
    EmbeddingModelKind::OpenAIAda02, EmbeddingModelKind::T5Base,
};
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::info;

use openai::OpenAI;
//use sentence_transformers::SentenceTransformerModels;

/// An enumeration of possible errors that can occur while generating text embeddings.
#[derive(Error, Debug)]
pub enum EmbeddingGeneratorError {
    /// An error that occurs when the requested model is not found.
    #[error("model `{0}` not found")]
    ModelNotFound(String),

    /// An error that occurs when generating embeddings using a model.
    #[error("model inference error: `{0}`")]
    ModelError(String),

    /// An error that occurs when loading a model.
    #[error("model loading error: `{0}`")]
    ModelLoadingError(String),

    /// An internal error that occurs during the operation.
    #[error("internal error: `{0}`")]
    InternalError(String),

    /// An error that occurs when the required configuration is missing for a model.
    #[error("configuration `{0}`, missing for model `{1}`")]
    ConfigurationError(String, String),
}

pub type EmbeddingGeneratorTS = Arc<dyn EmbeddingGenerator + Sync + Send>;

/// A trait that defines the interface for generating text embeddings.
#[async_trait]
pub trait EmbeddingGenerator {
    /// Generates text embeddings for the given texts using the specified model.
    ///
    /// # Arguments
    ///
    /// * `inputs` - A vector of strings for which embeddings are to be generated.
    /// * `model` - The name of the model to be used for generating embeddings.
    ///
    /// # Returns
    ///
    /// * A result containing a vector of embeddings if successful, or an `EmbeddingGeneratorError`
    ///   if an error occurs.
    async fn generate_embeddings(
        &self,
        inputs: Vec<String>,
    ) -> Result<Vec<Vec<f32>>, EmbeddingGeneratorError>;

    /// Returns the number of dimensions of the embeddings generated by the specified model.
    ///
    /// # Arguments
    ///
    /// * `model` - The name of the model for which the number of dimensions is to be retrieved.
    ///
    /// # Returns
    ///
    /// * A result containing the number of dimensions if successful, or an `EmbeddingGeneratorError`
    ///   if an error occurs (e.g., the model is not found).
    fn dimensions(&self) -> u64;

    async fn tokenize_encode(
        &self,
        inputs: Vec<String>,
    ) -> Result<Vec<Vec<u64>>, EmbeddingGeneratorError>;

    async fn tokenize_decode(
        &self,
        inputs: Vec<Vec<u64>>,
    ) -> Result<Vec<String>, EmbeddingGeneratorError>;
}

/// A struct that represents a router for generating text embeddings using different models.
///
/// This struct provides methods for generating text embeddings using various models.
/// It maintains a mapping between model names and their corresponding `EmbeddingGenerator`
/// implementations, allowing it to route embedding generation requests to the appropriate model.
pub struct EmbeddingRouter {
    router: DashMap<String, EmbeddingGeneratorTS>,

    model_names: Vec<String>,
}

impl EmbeddingRouter {
    pub fn new(config: Arc<server_config::ServerConfig>) -> Result<Self, EmbeddingGeneratorError> {
        let router: DashMap<String, EmbeddingGeneratorTS> = DashMap::new();
        let mut model_names = Vec::new();
        for model in config.available_models.clone() {
            model_names.push(model.model_kind.to_string());
            info!(
                "loading embedding model: {:?}",
                model.model_kind.to_string()
            );
            match model.model_kind {
                AllMiniLmL12V2 => {
                    let lm12_model = sentence_transformers::AllMiniLmL12V2::new()?;
                    router.insert(model.model_kind.to_string(), Arc::new(lm12_model));
                }
                AllMiniLmL6V2 => {
                    let lm6_model = sentence_transformers::AllMiniLmL6V2::new()?;
                    router.insert(model.model_kind.to_string(), Arc::new(lm6_model));
                }
                T5Base => {
                    let t5_base = sentence_transformers::T5Base::new()?;
                    router.insert(model.model_kind.to_string(), Arc::new(t5_base));
                }
                OpenAIAda02 => {
                    let openai_config = config.openai.clone().ok_or(
                        EmbeddingGeneratorError::ConfigurationError(
                            "openai".into(),
                            "openai_config".into(),
                        ),
                    )?;
                    let openai_ada02 = OpenAI::new(openai_config, model.clone())?;
                    router.insert(model.model_kind.to_string(), Arc::new(openai_ada02));
                }
                crate::EmbeddingModelKind::DPRModel => {
                    let dpr_model = DPREmbeddings::new()
                        .map_err(|e| EmbeddingGeneratorError::ModelLoadingError(e.to_string()))?;
                    router.insert(model.model_kind.to_string(), Arc::new(dpr_model));
                }
                _ => {
                    return Err(EmbeddingGeneratorError::InternalError(format!(
                        "model kind `{}` not supported",
                        model.model_kind
                    )))
                }
            }
        }
        Ok(Self {
            router,
            model_names,
        })
    }

    pub fn get_model(&self, name: &str) -> Result<EmbeddingGeneratorTS, EmbeddingGeneratorError> {
        let embedding_model = self
            .router
            .get(name)
            .ok_or(EmbeddingGeneratorError::ModelNotFound(name.into()))?;
        Ok(embedding_model.clone())
    }

    pub fn list_models(&self) -> Vec<String> {
        self.model_names.clone()
    }
}
