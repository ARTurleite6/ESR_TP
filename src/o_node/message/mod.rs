use serde::{Deserialize, Serialize};

pub mod query;
pub mod answer;

pub mod rtsp;

pub trait Message<T>: std::fmt::Debug + Clone + Serialize + for<'de> Deserialize<'de> {
    fn id(&self) -> u32;
    fn payload(&self) -> Option<&T>;
    fn check_id(&self, message: Self) -> bool { 
        return self.id() == message.id();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum Status {
    #[default]
    Ok,
    Error,
    Query,
}