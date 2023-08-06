use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};

thread_local! {
    static EMBEDDING_MODEL: SentenceEmbeddingsModel =
            SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
                .create_model()
                .unwrap();
}

pub async fn get_embedding(string: &str) -> anyhow::Result<Vec<f32>> {
    let string = string.to_owned();

    tokio::task::spawn_blocking(move || {
        Ok(EMBEDDING_MODEL
            .with(|model| model.encode(&[string]))
            .map(|mut v| v.pop().unwrap())?)
    })
    .await?
}

#[cfg(test)]
mod test {
    // use crate::data::{get_all_data, get_latest_data};

    #[tokio::test]
    async fn get_emb() -> anyhow::Result<()> {
        let emb = super::get_embedding("hello").await?;
        // println!("emb: {:?}", emb);
        println!("size: {}", emb.len());

        Ok(())
    }
}
