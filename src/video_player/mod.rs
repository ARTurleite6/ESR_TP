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
enum Message {
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

    fn update(message: &Message, client: &Arc<RwLock<Client>>, widgets: &Rc<VideoWidgets>) {
        match message {
            Message::Setup => {
                widgets.set_label_text("State: Ready");
                let mut lock = client.write().unwrap();
                let result = lock.setup();
                dbg!(&result);
                if result.is_err() {
                    dbg!("Error setting up");
                    widgets.set_label_text("State: Idle (Error setting up)");
                }
            }
            Message::Play => {
                widgets.set_label_text("State: Playing");
                if VideoPlayer::play(client, widgets).is_err() {
                    dbg!("Error playing video");
                    widgets.set_label_text("State: Idle (Error playing video)");
                }
                dbg!("Play");
            }
            Message::Teardown => {
                if client.write().unwrap().stop_transmition().is_err() {
                    dbg!("Error stopping transmission");
                    widgets.set_label_text("State: Error stopping transmission");
                } else {
                    widgets.set_label_text("State: Idle");
                }
            }
            _ => todo!(),
        }
    }

    fn store_file_cache(video: &[u8], session_id: u32) -> String {
        let path = format!("{}/{}.{}", CACHE_DIRECTORY, session_id, CACHE_EXTENSION);

        let mut file = std::fs::File::create(&path).expect("Error creating file");

        let size = file.write_all(video).expect("Error writing to cache");

        return path;
    }

    fn register_callback(
        client: &Arc<RwLock<Client>>,
        widgets: &Rc<VideoWidgets>,
        message: Message,
        button: &gtk::Button,
    ) {
        let client_clone = Arc::clone(&client);
        let widgets_clone = Rc::clone(&widgets);
        button.connect_clicked(move |_| {
            Self::update(&message, &client_clone, &widgets_clone);
        });
    }

    fn register_callbacks(
        client: Arc<RwLock<Client>>,
        widgets: Rc<VideoWidgets>,
        window: &ApplicationWindow,
    ) {
        VideoPlayer::register_callback(&client, &widgets, Message::Setup, widgets.setup_button());
        VideoPlayer::register_callback(&client, &widgets, Message::Play, widgets.play_button());
        VideoPlayer::register_callback(
            &client,
            &widgets,
            Message::Teardown,
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

            Self::register_callbacks(client, widgets, &window);

            window.show_all();
        });
    }

    fn play(
        client: &Arc<RwLock<Client>>,
        video_widget: &Rc<VideoWidgets>,
    ) -> Result<(), RequestError> {
        let mut lock = client.write().unwrap();

        let answer = lock.make_request(message::rtsp::RequestType::Play, 0)?;

        let session_id = lock.session_id();
        drop(lock);

        let (tx, rx) = MainContext::channel(gtk::glib::Priority::DEFAULT);

        let client_clone = Arc::clone(&client);
        thread::spawn(move || loop {
            let lock = client_clone.read().unwrap();

            if lock.is_stopped() {
                tx.send(None)
                    .expect("Error sending path to another channel");
                break;
            }

            let packet = lock.receive_rtp_packet();
            drop(lock);

            let data = packet.payload();

            let path = VideoPlayer::store_file_cache(&data, session_id);
            dbg!(&path);

            tx.send(Some(path))
                .expect("Error sending path to another channel");
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
