//! Carregamento e validação das definições JSON compartilhadas.

mod catalog;
mod loader;
mod types;
mod validate;

pub use catalog::Catalog;
pub use loader::{Loaded, load};
pub use types::*;
pub use validate::validate;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DefsError {
    #[error("falha ao ler {file}: {source}")]
    Json {
        file: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("definição inválida: {0}")]
    Invalid(String),
}
