#![allow(dead_code)]

pub mod bootstraper_node;
pub mod config;
pub mod std_node;
mod query_worker;
pub mod neighbour;
mod errors;

use std::fmt::Debug;

use config::{Configuration, NodeFunction};
use thiserror::Error;

use self::{bootstraper_node::BootstraperNode, std_node::StdNode, neighbour::Neighbour};

#[derive(Debug, Error)]
pub enum NodeCreationError {
    #[error("The file passed to create topology does not exist, topology: {0}")]
    InexistentTopology(String),
    #[error("Error binding socket: {0}")]
    ErrorBindingSocket(std::io::Error),
    #[error("Error connecting to bootstraper: {0}")]
    ErrorConnectingBootstraper(std::io::Error),
    #[error("Error deserializing the ip addresses")]
    ErrorDeserializingIpAddresses(bincode::Error),
}

pub fn create_node(configuration: Configuration) -> Result<Box<dyn Node>, NodeCreationError> {
    match configuration.node_function {
        NodeFunction::Bootstraper { .. } => BootstraperNode::from_configuration(configuration)
            .map(|node| Box::new(node) as Box<dyn Node>),
        NodeFunction::NonBootstraper { .. } => {
            StdNode::from_configuration(configuration).map(|node| Box::new(node) as Box<dyn Node>)
        }
    }
}

pub trait Node: Debug {
    fn from_configuration(configuration: Configuration) -> Result<Self, NodeCreationError>
    where
        Self: Sized;

    fn neighbours(&self) -> &[Neighbour];

    fn run(&self) -> Result<(), NodeCreationError>;
}
