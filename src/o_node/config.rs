use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Configuration {
    ///Port in which this server will be listening to
    pub port: u16,
    #[command(subcommand)]
    pub node_function: NodeFunction,
}

#[derive(Subcommand, Debug)]
pub enum NodeFunction {
    NonBootstraper {
        /// Node to communicate in order to get neighboors
        bootstraper_ip: String,
    },
    Bootstraper {
        /// File containing topology in order to serve as boostraper for other nodes
        topology: String,
        port: u16,
    },
}
