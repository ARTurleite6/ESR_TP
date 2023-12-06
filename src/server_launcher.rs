use clap::Parser;
use esr_lib::server::Server;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = "8554")]
    streaming_port: u16,

    #[clap(short, long, default_value = "8555")]
    metrics_port: u16,
}

fn main() {
    let args = Args::parse();

    Server::new(args.metrics_port, args.streaming_port)
        .expect("Error creating server")
        .run();
}
