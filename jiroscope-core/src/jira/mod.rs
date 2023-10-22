mod doc;
mod error;
mod issue;
mod project;
pub use doc::*;
pub use error::*;
pub use issue::*;
pub use project::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub start_at: usize,
    pub max_results: usize,
    pub total: usize,
    pub is_last: bool,
    pub values: Vec<T>,
}
