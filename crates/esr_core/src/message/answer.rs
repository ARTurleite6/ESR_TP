use super::{query::Query, Message, Status};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Answer<T> {
    id: u32,
    status: Status,
    payload: T,
}

impl<T: Clone + std::fmt::Debug + Serialize + for<'de> Deserialize<'de>> Message<T> for Answer<T> {
    fn id(&self) -> u32 {
        self.id
    }

    fn payload(&self) -> Option<&T> {
        Some(&self.payload)
    }

}

impl<T> Answer<T> {

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn from_message(message: Query, payload: T, status: Status) -> Self {
        Self {
            id: message.id(),
            status,
            payload,
        }
    }

    pub fn payload_mut(&mut self) -> &mut T {
        &mut self.payload
    }
}

