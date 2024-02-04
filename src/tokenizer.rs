use tiktoken_rs::p50k_base;

pub trait TokenCount {
    fn token_count(&self) -> usize;
}

impl TokenCount for &str {
    fn token_count(&self) -> usize {
        let bpe = p50k_base().unwrap();
        bpe.encode_with_special_tokens(self).len()
    }
}

impl TokenCount for String {
    fn token_count(&self) -> usize {
        self.as_str().token_count()
    }
}

pub trait EstimetedTokenCount {
    fn estimated_token_count(&self) -> usize;
}

impl EstimetedTokenCount for &str {
    fn estimated_token_count(&self) -> usize {
        self.chars().count().div_ceil(4)
    }
}

impl EstimetedTokenCount for String {
    fn estimated_token_count(&self) -> usize {
        self.as_str().estimated_token_count()
    }
}
