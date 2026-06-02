use async_trait::async_trait;

use crate::{domain::short_code::random_base62_code, ports::code_generator::CodeGenerator};

pub struct Base62CodeGenerator {
    length: usize,
}

impl Base62CodeGenerator {
    pub fn new(length: usize) -> Self {
        Self { length }
    }
}

#[async_trait]
impl CodeGenerator for Base62CodeGenerator {
    async fn generate(&self) -> String {
        random_base62_code(self.length)
    }
}
