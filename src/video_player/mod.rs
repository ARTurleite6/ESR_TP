#![allow(dead_code)]

mod client;
mod video_widgets;

use clap::Parser;
use gtk::glib::MainContext;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow};
use std::io::Write;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::message;

use self::client::{Client, RequestError};
use self::video_widgets::VideoWidgets;

const CACHE_DIRECTORY: &'static str = "tmp";
const CACHE_EXTENSION: &'static str = "Mjpeg";

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(short = 's', long, default_value = "0.0.0.0")]
    server_name: String,
    #[clap(short = 'p', long, default_value = "8554")]
    server_port: u16,
    #[clap(short, long, default_value = "5000")]
    rtp_port: u16,
    #[clap(short, long, default_value = "movie.Mjpeg")]
    video_file: String,
}

trait VideoPlayerComponent {
    type Init;

    fn from_init(init: &Self::Init) -> Self;
}

pub struct VideoPlayer;

#[derive(Debug, Clone, Copy)]
enum VideoPlayerAction {
    Play,
    Pause,
    Setup,
    Teardown,
}

impl VideoPlayer {
    pub fn run(init: Args) {
        let app = Application::builder()
            .application_id(&format!("video.streamer/{}", init.rtp_port))
            .build();

        Self::setup(&app, init);

        app.run_with_args::<&str>(&[]);
    }

    fn update(
        message: &VideoPlayerAction,
        client: &Arc<RwLock<Client>>,
        widgets: &Rc<VideoWidgets>,
    ) {
        match message {
            VideoPlayerAction::Setup => {
                widgets.set_label_text("State: Ready");
                let mut lock = client
                    .write()
                    .expect("Error acquiring the client's writing lock");
                if lock.setup().is_err() {
                    dbg!("Error setting up");
                    widgets.set_label_text("State: Idle (Error setting up)");
                }
            }
            VideoPlayerAction::Play => {
                widgets.set_label_text("State: Playing");
                if VideoPlayer::play(client, widgets).is_err() {
                    dbg!("Error playing video");
                    widgets.set_label_text("State: Idle (Error playing video)");
                }
                dbg!("Play");
            }
            VideoPlayerAction::Pause => {
                widgets.set_label_text("State: Paused");
                if client
                    .write()
                    .expect("Error acquiring the client's writing lock")
                    .pause()
                    .is_err()
                {
                    dbg!("Error pausing transmission");
                    widgets.set_label_text("State: Error pausing transmission");
                } else {
                    widgets.set_label_text("State: Pause");
                }
            }
            VideoPlayerAction::Teardown => {
                if client
                    .write()
                    .expect("Error acquiring the client's writing lock")
                    .stop_transmition()
                    .is_err()
                {
                    dbg!("Error stopping transmission");
                    widgets.set_label_text("State: Error stopping transmission");
                } else {
                    widgets.set_label_text("State: Idle");
                }
            }
        }
    }

    fn store_file_cache(video: &[u8], session_id: u32) -> std::io::Result<String> {
        let path = format!("{}/{}.{}", CACHE_DIRECTORY, session_id, CACHE_EXTENSION);

        let mut file = std::fs::File::create(&path)?;

        file.write_all(video)?;

        return Ok(path);
    }

    fn register_callback(
        client: &Arc<RwLock<Client>>,
        widgets: &Rc<VideoWidgets>,
        message: VideoPlayerAction,
        button: &gtk::Button,
    ) {
        let client_clone = Arc::clone(&client);
        let widgets_clone = Rc::clone(&widgets);
        button.connect_clicked(move |_| {
            Self::update(&message, &client_clone, &widgets_clone);
        });
    }

    fn register_callbacks(client: Arc<RwLock<Client>>, widgets: Rc<VideoWidgets>) {
        VideoPlayer::register_callback(
            &client,
            &widgets,
            VideoPlayerAction::Setup,
            widgets.setup_button(),
        );
        VideoPlayer::register_callback(
            &client,
            &widgets,
            VideoPlayerAction::Play,
            widgets.play_button(),
        );
        VideoPlayer::register_callback(
            &client,
            &widgets,
            VideoPlayerAction::Pause,
            widgets.pause_button(),
        );
        VideoPlayer::register_callback(
            &client,
            &widgets,
            VideoPlayerAction::Teardown,
            widgets.teardown_button(),
        );
    }

    fn setup(app: &Application, init: Args) {
        app.connect_activate(move |app| {
            let window = ApplicationWindow::builder()
                .application(app)
                .title("Video Streamer")
                .default_width(800)
                .default_height(600)
                .build();

            let widgets = Rc::new(VideoWidgets::new(&window));
            let client = Arc::new(RwLock::new(Client::from_init(&init)));
            dbg!(&client);

            Self::register_callbacks(client, widgets);

            window.show_all();
        });
    }

    fn play(
        client: &Arc<RwLock<Client>>,
        video_widget: &Rc<VideoWidgets>,
    ) -> Result<(), RequestError> {
        let mut lock = client.write().expect("Failed to acquire lock");

        let answer = lock.make_request(message::rtsp::RequestType::Play)?;
        if !answer.succeded() {
            return Err(RequestError::FailedRequest);
        }

        let session_id = lock.session_id();
        drop(lock);

        let (tx, rx) = MainContext::channel(gtk::glib::Priority::DEFAULT);

        let client_clone = Arc::clone(&client);
        thread::spawn(move || loop {
            let lock = client_clone.read().unwrap();

            if lock.is_stopped() {
                if let Err(error) = tx.send(None) {
                    println!("Error sending path to another channel {}", error);
                }
                break;
            }

            let packet = lock.receive_rtp_packet();
            drop(lock);

            let data = packet.payload();

            let path = VideoPlayer::store_file_cache(&data, session_id);
            dbg!(&path);
            match path {
                Ok(path) => {
                    tx.send(Some(path))
                        .expect("Error sending path to another channel");
                }
                Err(error) => {
                    println!("Error storing file {}", error)
                }
            }
        });

        let video_widgets_clone = Rc::clone(&video_widget);
        rx.attach(None, move |path| {
            video_widgets_clone.update_image(path.as_deref());
            while gtk::glib::MainContext::default().iteration(false) {}
            return gtk::glib::ControlFlow::Continue;
        });

        return Ok(());
    }
}
