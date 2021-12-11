use serde::{Deserialize};

#[derive(Deserialize, Debug)]
pub struct PagedResult<T> {
    pub total_count: u32,
    pub incomplete_results: bool,
    pub items: Vec<T>
}
