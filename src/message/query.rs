use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};

use crate::o_node::neighbour::Neighbour;

use super::{Message, Status};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileQuery {
    file: String,
    already_asked: Vec<Neighbour>
}

impl FileQuery {
    pub fn new(file: &str, already_asked: Vec<Neighbour>) -> Self {
        Self {
            file: file.to_string(),
            already_asked
        }
    }

    pub fn file(&self) -> &str {
        return &self.file;
    }

    pub fn visited_neighbour(&self, neighbour: &Neighbour) -> bool {
        return self.already_asked.contains(neighbour);
    }

    pub fn add_neighbour(&mut self, neighbour: Neighbour) {
        self.already_asked.push(neighbour);
    }
    
    pub fn add_neighbours(&mut self, neighbours: &[Neighbour]) {
        self.already_asked.extend_from_slice(neighbours);
    }

    pub fn is_first_node(&self) -> bool {
        return self.already_asked.is_empty();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum QueryType {
    #[default]
    Neighbours,
    File(FileQuery),
}

impl QueryType {
    pub fn file_query(&self) -> Option<&FileQuery> {
        match self {
            Self::File(args) => Some(&args),
            _ => None
        }
    }

    pub fn file_query_mut(&mut self) -> Option<&mut FileQuery> {
        match self {
            Self::File(args) => Some(args),
            _ => None
        }
    }
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

    pub fn new_file_query(file: &str, payload: Option<String>) -> Self {
        let mut rng = rand::thread_rng();

        let query_type = QueryType::File(FileQuery::new(file, Vec::new()));
        Self {
            id: rng.gen::<u32>(),
            query_type,
            status: Status::Query,
            payload,
        }
    }

    pub fn query_type(&self) -> &QueryType {
        return &self.query_type;
    }

    pub fn query_type_mut(&mut self) -> &mut QueryType {
        return &mut self.query_type;
    }

    pub fn query_file(&self) -> Option<&str> {
        return self.query_type.file_query().map(|query| query.file.as_str());
    }
}

