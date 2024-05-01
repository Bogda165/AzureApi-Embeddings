use std::error::Error;
use rust2vec::prelude::*;
use std::fs::File;
use std::io::BufReader;
use ndarray::Array1;
use std::time::Instant;

pub struct Embedding {
    Embeddings: Embeddings<SimpleVocab, NdArray>,
}

pub enum RequestType {
    Caption(String),
    Labels(Vec<String>),
    Full { caption: String, labels: Vec<String> }
}

impl Embedding {
    pub fn new() -> Self {
        Embedding {
            Embeddings: Embeddings::new(None, SimpleVocab::new(vec!["<UNK>".to_owned()]), NdArray(Default::default()), )
        }
    }

    pub fn get_embeddings(&mut self, path: &str) {
        let start = Instant::now();
        println!("Start loading Embeddings");
        let mut reader = BufReader::new(File::open(path).unwrap());
        self.Embeddings = Embeddings::read_text(&mut reader, true).unwrap();

        println!("Embeddings are loaded!!!\nTime: {:?}", start.elapsed());
    }

    fn prepare_text(text: &str) -> Vec<String> {
        let mut tokens: String = String::new();
        for i in text.chars() {
            if i.is_alphabetic() || i == ' ' {
                tokens.push(i);
            }
        }

        tokens = tokens.to_lowercase();
        tokens.split_whitespace().map(|s| s.to_string()).collect()
    }
    pub fn average_vector(&self, sentence: String) -> Vec<f32> {
        //println!("Sentence: {:?}", sentence);
        let words: Vec<String> = Self::prepare_text(&*sentence);
        //println!("After split: {:?}", words);
        let mut vector = vec![0.0; self.Embeddings.dims()];
        let mut count = 0;

        for word in words {
            if let Some(embedding) = self.Embeddings.embedding(word.as_str()) {
                for (i, value) in embedding.as_view().iter().enumerate() {
                    vector[i] += *value;
                }
                count += 1;
            }
        }

        if count > 0 {
            for value in &mut vector {
                *value /= count as f32;
            }
        }

        vector
    }

    pub fn cosine_similarity(vector1: &[f32], vector2: &[f32]) -> f32 {
        let start = Instant::now();
        println!("Start calculating similarity");

        let dot_product: f32 = vector1.iter().zip(vector2).map(|(a, b)| a * b).sum();
        let magnitude1: f32 = vector1.iter().map(|a| a.powi(2)).sum::<f32>().sqrt();
        let magnitude2: f32 = vector2.iter().map(|a| a.powi(2)).sum::<f32>().sqrt();

        println!("Similarity is calculated\nTime: {:?}", start.elapsed().as_secs());
        dot_product / (magnitude1 * magnitude2)
    }

    pub fn semantic_vector(&mut self, phrases: Vec<String>) -> Vec<f32> {
        let mut sum_vector = vec![0.0; self.Embeddings.dims()];
        let mut count = 0;

        for phrase in phrases {
            let vector = self.average_vector(phrase);
            for i in 0..vector.len() {
                sum_vector[i] += vector[i];
            }
            count += 1;
        }

        for i in 0..sum_vector.len() {
            sum_vector[i] /= count as f32;
        }

        sum_vector
    }

    pub fn main_vector(&mut self, request: RequestType) -> Vec<f32>{
        match request {
            RequestType::Caption(caption) => {
                self.average_vector(caption)
            }
            RequestType::Labels(labels) => {
                self.semantic_vector(labels)
            }
            RequestType::Full {caption, labels } => {
                // Change to formula from Anton code!
                self.semantic_vector(labels)
            }
        }
    }

    pub fn similarity_string(&mut self, phrase1: String, phrase2: String) -> f32 {
        let vector1 = self.average_vector(phrase1);
        let vector2 = self.average_vector(phrase2);

        Embedding::cosine_similarity(&vector1, &vector2)
    }
}
