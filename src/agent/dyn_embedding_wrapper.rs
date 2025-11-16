use std::sync::Arc;

use rig::embeddings::{Embedding, EmbeddingError, EmbeddingModel, embedding::EmbeddingModelDyn};

/// Wraps a `dyn EmbeddingModelDyn` so it can satisfy the `EmbeddingModel` trait.
///
/// The maximum batch size is expressed via the `MAX` const parameter, so callers can
/// align it with the underlying provider without hard-coding a single limit here.
#[derive(Clone)]
pub struct DynEmbeddingModelWrapper<const MAX: usize> {
    inner: Arc<dyn EmbeddingModelDyn>,
}

impl<const MAX: usize> DynEmbeddingModelWrapper<MAX> {
    pub fn new(inner: Arc<dyn EmbeddingModelDyn>) -> Self {
        Self { inner }
    }

    pub fn model(&self) -> Arc<dyn EmbeddingModelDyn> {
        Arc::clone(&self.inner)
    }
}

impl<const MAX: usize> EmbeddingModel for DynEmbeddingModelWrapper<MAX> {
    const MAX_DOCUMENTS: usize = MAX;

    fn ndims(&self) -> usize {
        self.inner.ndims()
    }

    fn embed_texts(
        &self,
        texts: impl IntoIterator<Item = String> + Send,
    ) -> impl std::future::Future<Output = Result<Vec<Embedding>, EmbeddingError>> + Send {
        let docs: Vec<String> = texts.into_iter().collect();
        let model = Arc::clone(&self.inner);

        async move { model.embed_texts(docs).await }
    }
}
