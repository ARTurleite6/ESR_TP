use super::{Status, Message, query::Query};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer<T> {
    id: u32,
    status: Status,
    payload: T,
}

impl<T: Clone + std::fmt::Debug + Serialize + for <'de> Deserialize<'de>> Message<T> for Answer<T> {
    fn id(&self) -> u32 {
        self.id
    }

    fn payload(&self) -> Option<&T> {
        Some(&self.payload)
    }
}

impl<T> Answer<T> {
    pub fn from_message(message: Query, payload: T, status: Status) -> Self {
        Self {
            id: message.id(),
            status,
            payload
        }
    }
}