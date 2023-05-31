use std::sync::Arc;

use crate::{
    persistence::{Chunk, Respository},
    text_splitters::TextSplitterTS,
    EmbeddingGeneratorTS, VectorChunk, VectorDBTS,
};
use anyhow::Result;
use tracing::info;

pub struct EmbeddingWorker {
    repository: Arc<Respository>,
    vectordb: VectorDBTS,
    embedding_generator: EmbeddingGeneratorTS,
    embedding_model: String,
    text_splitter: TextSplitterTS,
}

impl EmbeddingWorker {
    pub fn new(
        repository: Arc<Respository>,
        vectordb: VectorDBTS,
        embedding_generator: EmbeddingGeneratorTS,
        embedding_model: String,
        text_splitter: TextSplitterTS,
    ) -> Self {
        Self {
            repository,
            vectordb,
            embedding_generator,
            embedding_model,
            text_splitter,
        }
    }

    pub async fn run_once(&self) -> Result<()> {
        let content_list = self.repository.not_indexed_content().await?;
        for content in content_list {
            let text = content.text;
            let index = content.index_name;
            let splitted_texts = self.text_splitter.split(&text, 1000, 0).await?;
            let embeddings = self
                .embedding_generator
                .generate_embeddings(splitted_texts.clone(), self.embedding_model.to_string())
                .await?;
            let mut chunks: Vec<Chunk> = Vec::new();
            let mut vector_chunks: Vec<VectorChunk> = Vec::new();
            for (text, embedding) in splitted_texts.iter().zip(embeddings.iter()) {
                let chunk = Chunk::new(text.to_string(), content.id.clone());
                let vector_chunk =
                    VectorChunk::new(chunk.chunk_id.clone(), text.to_string(), embedding.to_vec());
                chunks.push(chunk);
                vector_chunks.push(vector_chunk);
            }
            self.vectordb.add_embedding(&index, vector_chunks).await?;
            info!("adding to index {}", index);
            self.repository
                .create_chunks(content.id, chunks, index.clone())
                .await?;
        }
        Ok(())
    }
}
