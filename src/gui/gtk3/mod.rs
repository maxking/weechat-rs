use gtk::prelude::*;
use gtk::{self, Button, Window, WindowType};

pub fn launch() {
    gtk::init().unwrap_or_else(|_| panic!("weechat-rs: failed to initialize GTK."));

    let window = Window::new(WindowType::Toplevel);
    window.set_title("First GTK+ program");
    window.set_default_size(350, 70);

    let button = Button::new_with_label("Click me!");
    window.add(&button);
    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    button.connect_clicked(|_| {
        println!("Clicked the button");
    });

    button.connect_clicked(|but| {
        but.set_label("I've been clicked!");
    });

    gtk::main();
}
