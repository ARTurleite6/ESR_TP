#![allow(dead_code)]
use clap::Parser;
use gtk::glib::MainContext;
use gtk::{prelude::*, Image};
use gtk::{Application, ApplicationWindow};
use rand::Rng;
use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::thread;

use crate::o_node::message;
use crate::o_node::message::rtp::RtpPacket;
use crate::o_node::message::rtsp::{RtspRequest, RtspResponse};

const CACHE_DIRECTORY: &'static str = "tmp";
const CACHE_EXTENSION: &'static str = "Mjpeg";

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(short = 's', long, default_value = "localhost")]
    server_name: String,
    #[clap(short = 'p', long, default_value = "8554")]
    server_port: u16,
    #[clap(short, long, default_value = "5000")]
    rtp_port: u16,
    #[clap(short, long, default_value = "movie.Mjpeg")]
    video_file: String,
}

pub struct VideoPlayer;

trait VideoPlayerComponent {
    type Init;

    fn from_init(init: &Self::Init) -> Self;
}

struct VideoWidgets {
    play_button: gtk::Button,
    setup_button: gtk::Button,
    pause_button: gtk::Button,
    teardown_button: gtk::Button,
    image_widget: Image,
    label: gtk::Label,
}

impl VideoWidgets {
    pub fn new(window: &ApplicationWindow) -> Self {
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let image = Image::new();
        image.set_vexpand(true);
        vbox.append(&image);

        let label = gtk::Label::new(Some("State: Idle"));

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let play_button = gtk::Button::with_label("Play");
        hbox.append(&play_button);

        let setup_button = gtk::Button::with_label("Setup");
        hbox.append(&setup_button);

        let pause_button = gtk::Button::with_label("Pause");
        hbox.append(&pause_button);

        let teardown_button = gtk::Button::with_label("Teardown");
        hbox.append(&teardown_button);

        vbox.append(&hbox);
        vbox.append(&label);

        window.set_child(Some(&vbox));

        Self {
            image_widget: image,
            play_button,
            setup_button,
            pause_button,
            teardown_button,
            label,
        }
    }
}

#[derive(Debug)]
struct ServerConnection {
    server_socket: TcpStream,
    udp_socket: UdpSocket,
    session_id: Option<u32>,
    stop_transmission: bool,
}

#[derive(Debug, Default)]
pub struct Client {
    server_name: String,
    server_port: u16,
    rtp_port: u16,
    video_file: String,
    server_connection: Option<ServerConnection>,
}

impl VideoPlayerComponent for Client {
    type Init = Args;

    fn from_init(init: &Self::Init) -> Self {
        Self::new(
            init.server_name.clone(),
            init.server_port,
            init.rtp_port,
            init.video_file.clone(),
        )
    }
}

impl Client {
    fn new(server_name: String, server_port: u16, rtp_port: u16, video_file: String) -> Self {
        Self {
            server_name,
            server_port,
            rtp_port,
            video_file,
            ..Default::default()
        }
    }

    fn stop_transmition(&mut self) {
        self.server_connection
            .as_mut()
            .expect("Expected server connection at this point")
            .stop_transmission = true;
    }

    fn setup(&mut self) {
        let mut rng = rand::thread_rng();

        let seq_number = rng.gen();

        let message = RtspRequest::new(
            message::rtsp::RequestType::Setup,
            self.video_file.clone(),
            seq_number,
            self.rtp_port,
        );

        let udp_socket =
            UdpSocket::bind(("127.0.0.1", self.rtp_port)).expect("Error binding rtp socket");

        let server_socket = TcpStream::connect((self.server_name.as_str(), self.server_port))
            .expect("Error connecting to server");

        self.server_connection = Some(ServerConnection {
            server_socket,
            udp_socket,
            session_id: None,
            stop_transmission: false,
        });

        self.send_rtps_packet(message);

        let response = self.receive_rtps_packet();

        self.server_connection.as_mut().unwrap().session_id = Some(response.session_id());
    }

    fn receive_rtp_packet(&self) -> RtpPacket {
        let mut buffer_size = [0; 8];

        let udp_socket = &self.server_connection.as_ref().unwrap().udp_socket;

        udp_socket
            .peek(&mut buffer_size)
            .expect("Error geetting size of buffer");

        let size: u64 = bincode::deserialize(&buffer_size).expect("Error deserializing size");

        let mut buffer = vec![0; (size + 8) as usize];

        let n = udp_socket
            .recv(&mut buffer)
            .expect("Error receiving packet");

        return bincode::deserialize(&buffer[8..n]).expect("Error deserializing packet");
    }

    fn send_rtps_packet(&mut self, packet: RtspRequest) {
        self.server_connection
            .as_mut()
            .unwrap()
            .server_socket
            .write_all(&bincode::serialize(&packet).expect("Error serializing packet"))
            .expect("Error sending packet to server");
    }

    fn receive_rtps_packet(&mut self) -> RtspResponse {
        let mut buffer = [0; 1024];

        self.server_connection
            .as_mut()
            .unwrap()
            .server_socket
            .read(&mut buffer)
            .expect("Error receiving packet from server");

        return bincode::deserialize(&buffer).expect("Error deserializing packet");
    }
}

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
            .application_id("video.streamer")
            .build();

        Self::setup(&app, init);

        app.run_with_args::<&str>(&[]);
    }

    fn update(message: &Message, client: &Arc<RwLock<Client>>, widgets: &Rc<VideoWidgets>) {
        match message {
            Message::Setup => {
                widgets.label.set_text("State: Ready");
                let mut lock = client.write().unwrap();
                lock.setup();
            }
            Message::Play => {
                widgets.label.set_text("State: Playing");
                VideoPlayer::play(client, widgets);
                dbg!("Play");
            }
            Message::Teardown => {
                widgets.label.set_text("Status: Idle");
                client.write().unwrap().stop_transmition();
            }
            _ => todo!(),
        }
    }

    fn store_file_cache(video: &[u8], session_id: u32) -> String {
        let path = format!("{}/{}.{}", CACHE_DIRECTORY, session_id, CACHE_EXTENSION);

        let mut file = std::fs::File::create(&path).expect("Error creating file");

        let size = file.write_all(video).expect("Error writing to cache");

        dbg!(size);

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

    fn register_callbacks(client: Arc<RwLock<Client>>, widgets: Rc<VideoWidgets>) {
        VideoPlayer::register_callback(&client, &widgets, Message::Setup, &widgets.setup_button);
        VideoPlayer::register_callback(&client, &widgets, Message::Play, &widgets.setup_button);
        VideoPlayer::register_callback(
            &client,
            &widgets,
            Message::Teardown,
            &widgets.teardown_button,
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

            Self::register_callbacks(client, widgets);

            window.present();
        });
    }

    fn play(client: &Arc<RwLock<Client>>, video_widget: &Rc<VideoWidgets>) {
        let mut lock = client.write().unwrap();

        let request = RtspRequest::new(
            message::rtsp::RequestType::Play,
            lock.video_file.clone(),
            0,
            lock.rtp_port,
        );

        let server_connection = lock.server_connection.as_mut().unwrap();

        let request = bincode::serialize(&request).expect("Error serializing packet");

        let tcp_socket = &mut server_connection.server_socket;

        tcp_socket.write_all(&request).unwrap();

        let mut buffer = [0; 1024];

        tcp_socket.read(&mut buffer).unwrap();

        let answer: RtspResponse =
            bincode::deserialize(&buffer).expect("Error deserializing packet");

        let session_id = server_connection.session_id.unwrap();
        drop(lock);

        let (tx, rx) = MainContext::channel(gtk::glib::Priority::DEFAULT);

        let client_clone = Arc::clone(&client);
        thread::spawn(move || loop {
            let lock = client_clone.read().unwrap();

            if lock.server_connection.as_ref().unwrap().stop_transmission {
                tx.send(None)
                    .expect("Error sending path to another channel");
                break;
            }

            let packet = lock.receive_rtp_packet();
            drop(lock);

            let data = packet.payload();

            dbg!(data.len());

            let path = VideoPlayer::store_file_cache(&data, session_id);
            dbg!(&path);

            tx.send(Some(path))
                .expect("Error sending path to another channel");
        });

        let video_widgets_clone = Rc::clone(&video_widget);
        rx.attach(None, move |path| {
            match path {
                Some(path) => video_widgets_clone.image_widget.set_from_file(Some(&path)),
                None => video_widgets_clone.image_widget.clear(),
            };
            while gtk::glib::MainContext::default().iteration(false) {}
            return gtk::glib::ControlFlow::Continue;
        });
    }
}
