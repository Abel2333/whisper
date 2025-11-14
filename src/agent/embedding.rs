use std::{collections::HashMap, sync::Arc};

use futures::stream;
use rig::{
    Embed,
    embeddings::{EmbedError, embedding::EmbeddingModelDyn, to_texts},
};

pub struct EmbeddingsBuilderDyn<T>
where
    T: Embed,
{
    model: Arc<dyn EmbeddingModelDyn + 'static>,
    documents: Vec<(T, Vec<String>)>,
}

impl<T> EmbeddingsBuilderDyn<T>
where
    T: Embed,
{
    pub fn new(model: Arc<dyn EmbeddingModelDyn + 'static>) -> Self {
        Self {
            model,
            documents: vec![],
        }
    }

    pub fn document(mut self, document: T) -> Result<Self, EmbedError> {
        let texts = to_texts(&document)?;
        self.documents.push((document, texts));
        Ok(self)
    }

    pub fn documents(self, documents: impl IntoIterator<Item = T>) -> Result<Self, EmbedError> {
        let builder = documents
            .into_iter()
            .try_fold(self, |builder, doc| builder.document(doc))?;

        Ok(builder)
    }
}

impl <T> EmbeddingsBuilderDyn<T>
    where 
    T:Embed + Send,
{
    pub async fn build(self)->Result<Vec<(T, OneOrMany<Embedding>)>, EmbeddingError> {
        use futures::stream::TryStreamExt;

        // Store the documents and their texts in a HashMap for easy access.
        let mut docs = HashMap::new();
        let mut texts = Vec::new();

        for (i, (doc, doc_texts)) in self.documents.into_iter().enumerate() {
            docs.insert(i, doc);
            texts.push((i, doc_texts));
        }

        let mut embeddings = stream::iter(texts.into_iter().map(move |text| (i)));
    }
}
