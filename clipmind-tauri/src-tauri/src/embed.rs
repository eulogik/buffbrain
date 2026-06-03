use anyhow::{Context, Result};
use ort::session::Session;
use ort::value::Tensor;
use std::path::Path;
use std::sync::Mutex;
use tokenizers::Tokenizer;
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_resources() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources")
    }

    #[test]
    fn test_embedder_creates_valid_embeddings() {
        let embedder = Embedder::new(&test_resources()).expect("failed to load model");
        let emb = embedder.embed("Hello world, this is a test sentence.").expect("embedding failed");
        assert_eq!(emb.len(), 384, "embedding should be 384-dimensional");
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 0.001, "embedding should be L2-normalized (norm={})", norm);
    }

    #[test]
    fn test_similar_texts_have_higher_similarity() {
        let embedder = Embedder::new(&test_resources()).expect("failed to load model");
        let a = embedder.embed("I love programming in Rust").unwrap();
        let b = embedder.embed("Rust programming is my favorite").unwrap();
        let c = embedder.embed("The weather is nice today").unwrap();

        let sim_ab: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let sim_ac: f32 = a.iter().zip(c.iter()).map(|(x, y)| x * y).sum();

        assert!(sim_ab > sim_ac, "similar texts should have higher cosine similarity (ab={}, ac={})", sim_ab, sim_ac);
    }

    #[test]
    fn test_empty_text_returns_zero_vector() {
        let embedder = Embedder::new(&test_resources()).expect("failed to load model");
        let emb = embedder.embed("").expect("embedding failed for empty text");
        assert_eq!(emb.len(), 384);
        assert!(emb.iter().all(|&x| x == 0.0), "empty text should return zero vector");
    }
}

const MAX_LENGTH: usize = 256;

pub struct Embedder {
    session: Mutex<Session>,
    tokenizer: Mutex<Tokenizer>,
}

impl Embedder {
    pub fn new(resources_dir: &Path) -> Result<Self> {
        let model_path = resources_dir.join("model_quantized.onnx");
        let tokenizer_path = resources_dir.join("tokenizer.json");

        let session = Session::builder()
            .context("failed to create ONNX Runtime session builder")?
            .commit_from_file(&model_path)
            .with_context(|| format!("failed to load model from {:?}", model_path))?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("failed to load tokenizer from {:?}: {e}", tokenizer_path))?;

        Ok(Self {
            session: Mutex::new(session),
            tokenizer: Mutex::new(tokenizer),
        })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        if text.trim().is_empty() {
            return Ok(vec![0.0f32; 384]);
        }

        let tokenizer = self.tokenizer.lock().unwrap();
        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| anyhow::anyhow!("tokenization failed: {e}"))?;

        let tokens = encoding.get_ids();
        let attention = encoding.get_attention_mask();
        let type_ids = encoding.get_type_ids();

        let seq_len = tokens.len().min(MAX_LENGTH);

        let mut input_ids = Vec::with_capacity(seq_len);
        let mut att_mask = Vec::with_capacity(seq_len);
        let mut tok_type = Vec::with_capacity(seq_len);
        for i in 0..seq_len {
            input_ids.push(tokens[i] as i64);
            att_mask.push(attention[i] as i64);
            tok_type.push(type_ids[i] as i64);
        }
        drop(tokenizer);

        let shape = vec![1_i64, seq_len as i64];

        let input_ids_t = Tensor::from_array((shape.clone(), input_ids))
            .context("failed to create input_ids tensor")?;
        let att_mask_t = Tensor::from_array((shape.clone(), att_mask.clone()))
            .context("failed to create attention_mask tensor")?;
        let tok_type_t = Tensor::from_array((shape, tok_type))
            .context("failed to create token_type_ids tensor")?;

        let mut session = self.session.lock().unwrap();
        let outputs = session
            .run(ort::inputs![
                "input_ids" => input_ids_t,
                "attention_mask" => att_mask_t,
                "token_type_ids" => tok_type_t,
            ])
            .context("ONNX inference failed")?;

        let output = &outputs["last_hidden_state"];
        let (_shape, data) = output
            .try_extract_tensor::<f32>()
            .context("failed to extract output tensor")?;

        let (_batch, seq, dim) = if _shape.len() == 3 {
            (_shape[0] as usize, _shape[1] as usize, _shape[2] as usize)
        } else {
            return Err(anyhow::anyhow!("unexpected output shape: {:?}", _shape));
        };

        let mut embedding = vec![0.0f32; dim];
        let mut mask_sum = 0.0f32;

        for s in 0..seq {
            let mask_val = att_mask[s] as f32;
            if mask_val == 0.0 {
                continue;
            }
            mask_sum += mask_val;
            for d in 0..dim {
                embedding[d] += data[s * dim + d];
            }
        }

        if mask_sum > 0.0 {
            for d in 0..dim {
                embedding[d] /= mask_sum;
            }
        }

        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for val in embedding.iter_mut() {
                *val /= norm;
            }
        }

        Ok(embedding)
    }
}
