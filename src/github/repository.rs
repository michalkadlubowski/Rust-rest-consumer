use serde::{Deserialize};

#[derive(Deserialize, Debug, Clone)]
pub struct  Owner {
    pub login: String
}

#[derive(Deserialize, Debug, Clone)]
pub struct Repository {
    pub name: String,
    pub owner: Owner
}

