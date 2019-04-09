extern crate libc;
extern crate ncurses;

mod weechat;

use std::ffi::CString;
use std::ptr;
use std::char;

use ncurses::*;


fn main() {
    // let mut session = weechat::RelaySession::new("127.0.0.1:8100".to_string());
    // println!("Connected to Weechat Server.");
    // let c_password = CString::new("hello").unwrap();
    // unsafe {
    //     weechat::weechat_relay_cmd_init(session, c_password.as_ptr(), ptr::null_mut());
    //     weechat::weechat_relay_cmd_info(session, ptr::null_mut(), CString::new("version").unwrap().as_ptr());
    // }
    initscr();
    raw();

    // allow extended keyboard.
    keypad(stdscr(), true);

    // Prompt for a character.
    addstr("Enter a character: ");
    let ch = getch();
    if ch == KEY_F1 {
        attron(A_BOLD() | A_BLINK());
        addstr("\nF1");
        attroff(A_BOLD() | A_BLINK());
        addstr(" pressed");
    } else {
        addstr("\nKey pressed: ");
        attron(A_BOLD() | A_BLINK());
        addstr(format!("{}\n", char::from_u32(ch as u32).expect("Invalid char")).as_ref());;
        attroff(A_BOLD() | A_BLINK());
    }

    refresh();
    getch();
    endwin();
}
