use clap::Parser;
use esr_lib::server::Server;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = "8554")]
    port: u16,
}

fn main() {
    let args = Args::parse();

    Server::new(args.port).run();
}
