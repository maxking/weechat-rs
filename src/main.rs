extern crate gtk;
extern crate gdk;
extern crate websocket;
extern crate byte;

mod gui {
    pub mod gtk3;
}

mod weechat{
    pub mod client;
}


fn main() {
    weechat::client::connect();
}
