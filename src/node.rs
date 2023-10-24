use clap::Parser;
use esr_lib::o_node::{config::Configuration, NodeCreationError, create_node};

fn main() -> Result<(), NodeCreationError> {
    let config = Configuration::parse();

    match create_node(config) {
        Ok(node) => {
            dbg!(&node);
            node.run()
        },
        Err(err) => println!("Error creating node: {:?}", err),
    };

    Ok(())
}