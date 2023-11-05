use gio::prelude::*;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Image, Label, Orientation};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::rc::Rc;

use crate::o_node::message::rtsp::{RequestType, RtspRequest};

pub struct Client {
    server_name: String,
    server_port: u16,
    rtp_port: u16,
    video_file: String,
}

impl Client {
    pub fn new(server_name: String, server_port: u16, rtp_port: u16, video_file: String) -> Self {
        Self {
            server_name,
            server_port,
            rtp_port,
            video_file,
        }
    }

    pub fn build_ui(&self, application: &Application) {
        // Create the main window
        let window = ApplicationWindow::new(application);
        window.set_title("Video Player");
        window.set_default_size(800, 600);

        // Create a vertical box to hold the widgets
        let vbox = gtk::Box::new(Orientation::Vertical, 5);
        window.add(&vbox);

        // Create an Image widget to display the video
        let video_image = Image::new();
        vbox.pack_start(&video_image, true, true, 0);

        // Create buttons for Play, Pause, Setup, and Teardown
        let button_play = Button::with_label("Play");
        let button_pause = Button::with_label("Pause");
        let button_setup = Button::with_label("Setup");
        let button_teardown = Button::with_label("Teardown");

        // Create labels to display messages
        let label_status = Rc::new(Label::new(Some("Status: Stopped")));

        // Pack buttons and labels into a horizontal box
        let hbox = gtk::Box::new(Orientation::Horizontal, 5);
        hbox.pack_start(&button_play, false, false, 0);
        hbox.pack_start(&button_pause, false, false, 0);
        hbox.pack_start(&button_setup, false, false, 0);
        hbox.pack_start(&button_teardown, false, false, 0);

        // Add buttons and labels to the vertical box
        vbox.pack_start(&hbox, false, false, 0);
        vbox.pack_start(&*label_status, false, false, 0);

        let label_status_clone = Rc::clone(&label_status);
        // Connect button signals
        button_play.connect_clicked(move |_| {
            //label_status.set_text("Status: Playing");
            label_status_clone.set_text("Status: Playing");
            // Add code to play the video here
        });

        let label_status_clone = Rc::clone(&label_status);
        button_pause.connect_clicked(move |_| {
            label_status_clone.set_text("Status: Paused");
            // Add code to pause the video here
        });

        let label_status_clone = Rc::clone(&label_status);
        let server_name = self.server_name.clone();
        let server_port = self.server_port;
        let file_name = self.video_file.clone();
        button_setup.connect_clicked(move |_| {
            println!("Stup");
            label_status_clone.set_text("Status: Setup");
        });

        let label_status_clone = Rc::clone(&label_status);
        button_teardown.connect_clicked(move |_| {
            label_status_clone.set_text("Status: Teardown");
            // Add code to teardown the video here
        });

        // Show all widgets
        window.show_all();
        window.set_application(Some(application))
    }
}
