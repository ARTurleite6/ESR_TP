use clap::Parser;
use esr_lib::client::{Args, VideoPlayer};

fn main() {
    let args = Args::parse();

    VideoPlayer::run(args);
}
