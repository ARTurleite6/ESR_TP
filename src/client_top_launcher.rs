use clap::Parser;
use esr_lib::video_player::{Args, VideoPlayer};

fn main() {
    let args = Args::parse();

    println!("Hello World");
    VideoPlayer::run(args);
}
