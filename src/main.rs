extern crate gdk;
extern crate gtk;
extern crate websocket;
#[macro_use]
extern crate nom;

mod gui {
    pub mod gtk3;
}

mod weechat {
    pub mod client;
    pub mod parser;
}

fn main() {
    weechat::client::connect();
}
