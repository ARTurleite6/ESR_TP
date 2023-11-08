#![allow(dead_code)]
use std::cell::RefCell;
use std::io::{Read, Write};
use std::net::{TcpStream, UdpSocket};
use std::rc::Rc;
use clap::Parser;
use gtk::{prelude::*, Image};
use gtk::{Application, ApplicationWindow};
use rand::Rng;

use crate::o_node::message;
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

        //let video = gtk::Video::new();
        //video.set_vexpand(true);
        //video.set_autoplay(true);
        //vbox.append(&video);

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

    fn setup(&mut self) {
        let mut rng = rand::thread_rng();

        let seq_number = rng.gen();

        let message = RtspRequest::new(
            message::rtsp::RequestType::Setup,
            self.video_file.clone(),
            seq_number,
            self.rtp_port,
        );

        dbg!(&message);

        let udp_socket =
            UdpSocket::bind(("127.0.0.1", self.rtp_port)).expect("Error binding rtp socket");

        let server_socket = TcpStream::connect((self.server_name.as_str(), self.server_port))
            .expect("Error connecting to server");

        self.server_connection = Some(ServerConnection {
            server_socket,
            udp_socket,
            session_id: None
        });

        self.send_rtps_packet(message);

        let response = self.receive_rtps_packet();

        self.server_connection.as_mut().unwrap().session_id = Some(response.session_id());

        dbg!(&response);
    }

    fn play(&mut self, image_widget: &Image) {
        let server_connection = self.server_connection.as_mut().unwrap();

        let request = RtspRequest::new(
            message::rtsp::RequestType::Play,
            self.video_file.clone(),
            0,
            self.rtp_port,
        );

        let request = bincode::serialize(&request).expect("Error serializing packet");

        let tcp_socket = &mut server_connection.server_socket;

        tcp_socket.write_all(&request).unwrap();

        let mut buffer = [0; 1024];

        tcp_socket.read(&mut buffer).unwrap();

        let answer: RtspResponse = bincode::deserialize(&buffer).expect("Error deserializing packet");

        dbg!(&answer);

        loop {
            let mut buffer = [0; 1024];

            let n = server_connection
                .udp_socket
                .recv(&mut buffer)
                .expect("Error receiving packet");

            let path = VideoPlayer::store_file_cache(&buffer[..n], server_connection.session_id.unwrap());

            image_widget.set_from_file(Some(path));

            dbg!(&buffer);
        }
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

#[derive(Debug)]
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

    fn update(message: Message, client: &Rc<RefCell<Client>>, widgets: &Rc<VideoWidgets>) {
        match message {
            Message::Setup => {
                widgets.label.set_text("State: Ready");
                client.borrow_mut().setup();
            }
            Message::Play => {
                widgets.label.set_text("State: Playing");
                client.borrow_mut().play(&widgets.image_widget);
            }
            _ => todo!(),
        }
    }

    fn store_file_cache(video: &[u8], session_id: u32) -> String {

        let path = format!("{}/{}.{}", CACHE_DIRECTORY, session_id, CACHE_EXTENSION);

        let mut file = std::fs::File::create(&path).expect("Error creating file");

        file.write(video).expect("Error writing to cache");

        return path;
    }

    fn register_callbacks(client: Rc<RefCell<Client>>, widgets: Rc<VideoWidgets>) {
        let client_clone = Rc::clone(&client);
        let widgets_clone = Rc::clone(&widgets);
        widgets.setup_button.connect_clicked(move |_| {
            let message = Message::Setup;
            Self::update(message, &client_clone, &widgets_clone);
        });

        let client_clone = Rc::clone(&client);
        let widgets_clone = Rc::clone(&widgets);
        widgets.play_button.connect_clicked(move |_| {
            let message = Message::Play;
            Self::update(message, &client_clone, &widgets_clone);
        });
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
            let client = Rc::new(RefCell::new(Client::from_init(&init)));

            Self::register_callbacks(client, widgets);

            window.present();
        });
    }
}