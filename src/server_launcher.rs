use clap::Parser;
use esr_lib::server::Server;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short, long, default_value = "8554")]
    port: u16,
    #[clap(short, long, default_value = "[movie.Mjpeg]")]
    videos: Vec<String>,
}

fn main() {
    let args = Args::parse();

    Server::new(args.port, args.videos).run();
}
