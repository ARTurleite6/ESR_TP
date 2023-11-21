use rand::Rng;
use serde::{Deserialize, Serialize};

use super::{Message, Status};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum QueryType {
    #[default]
    Neighbours,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Query {
    id: u32,
    query_type: QueryType,
    status: Status,
    payload: Option<String>,
}

impl Message<String> for Query {
    fn id(&self) -> u32 {
        self.id
    }

    fn payload(&self) -> Option<&String> {
        self.payload.as_ref()
    }
}

impl Query {
    pub fn new(query_type: QueryType, payload: Option<String>) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            id: rng.gen::<u32>(),
            query_type,
            status: Status::Query,
            payload,
        }
    }
}

