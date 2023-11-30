use gtk::{Image, ApplicationWindow, prelude::{BoxExt, ContainerExt, LabelExt, ImageExt}};

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
        image.set_from_file(Some("tmp/763082.Mjpeg"));
        vbox.pack_start(&image, true, true, 0);

        let label = gtk::Label::new(Some("State: Idle"));

        let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        let play_button = gtk::Button::with_label("Play");
        hbox.pack_start(&play_button, false, false, 0);

        let setup_button = gtk::Button::with_label("Setup");
        hbox.pack_start(&setup_button, false, false, 0);

        let pause_button = gtk::Button::with_label("Pause");
        hbox.pack_start(&pause_button, false, false, 0);

        let teardown_button = gtk::Button::with_label("Teardown");
        hbox.pack_start(&teardown_button, false, false, 0);

        vbox.pack_start(&hbox, false, false, 0);
        vbox.pack_start(&label, false, false, 0);

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
