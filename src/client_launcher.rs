use esr_lib::client::Client;
use gio::prelude::*;
use gtk::Application;

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
    #[clap(short = 's', long, default_value = "localhost")]
    server_name: String,
    #[clap(short = 'p', long, default_value = "8554")]
    server_port: u16,
    #[clap(short, long, default_value = "5000")]
    rtp_port: u16,
    #[clap(short, long, default_value = "movie.Mjpeg")]
    video_file: String,
}

fn main() {
    let args = Args::parse();

    dbg!(&args);

    let client = Client::new(
        args.server_name,
        args.server_port,
        args.rtp_port,
        args.video_file,
    );

    let application = Application::new(Some("com.example.video_player"), Default::default());

    application.connect_activate(move |app| {
        client.build_ui(app);
    });

    application.run_with_args(&[""]);

    // Initialize the GTK application
}
