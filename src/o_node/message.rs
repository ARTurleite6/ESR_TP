use rand::Rng;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Ok,
    Error,
    Query(QueryType),
    Answer,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum QueryType {
    Neighbours,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message<T: Serialize + Deserialize<'_>> {
    id: u32,
    status: Status,
    payload: Option<T>,
}

impl Message {
    pub fn query(payload: String, query_type: QueryType) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            id: rng.gen(),
            status: Status::Query(query_type),
            payload,
        }
    }

    pub fn answer(id: u32, payload: String, status: Status<T>) -> Self {
        Self { id, status, payload }
    }

    pub fn ok(&self) -> bool {
        self.status == Status::Ok
    }

    pub fn payload(&self, payload: String) -> &str {
        &self.payload
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn check_id(&self, id: u32) -> bool {
        self.id == id
    }
}