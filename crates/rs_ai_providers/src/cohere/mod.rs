pub mod provider;
pub mod rerank;

pub use provider::CohereProvider;
pub use rerank::CohereRerankingModel;

pub const RERANK_MODEL: &str = "rerank-v3.5";
