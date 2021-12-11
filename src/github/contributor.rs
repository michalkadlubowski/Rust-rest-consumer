use serde::{Deserialize};

#[derive(Deserialize, Debug)]
pub struct Contributor {
    pub login: String,
    pub contributions: u32
}
