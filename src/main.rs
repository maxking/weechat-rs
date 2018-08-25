extern crate gtk;
extern crate gdk;
extern crate websocket;
extern crate byte;
#[macro_use] extern crate nom;

mod gui {
    pub mod gtk3;
}

mod weechat{
    pub mod client;
    pub mod parser;
}


fn main() {
    weechat::client::connect();
}
