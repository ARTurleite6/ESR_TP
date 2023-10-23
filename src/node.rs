use clap::Parser;
use esr_lib::o_node::{config::Configuration, Node, NodeCreationError};

fn main() -> Result<(), NodeCreationError> {
    let config = Configuration::parse();


    let node = Node::from_configuration(config)?;

    node.run();

    Ok(())
}