use gtk::{prelude::*, ApplicationWindow, Image};

pub struct VideoWidgets {
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

    pub fn play_button(&self) -> &gtk::Button {
        &self.play_button
    }

    pub fn setup_button(&self) -> &gtk::Button {
        &self.setup_button
    }

    pub fn teardown_button(&self) -> &gtk::Button {
        &self.teardown_button
    }

    pub fn pause_button(&self) -> &gtk::Button {
        &self.pause_button
    }

    pub fn set_label_text(&self, text: &str) {
        self.label.set_text(text);
    }

    pub fn update_image(&self, image_path: Option<&str>) {
        if image_path.is_some() {
            self.image_widget.set_from_file(image_path);
        } else {
            self.image_widget.clear();
        }
    }
}
