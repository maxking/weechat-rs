use libc::{c_int, c_void, size_t};

use std::net::{TcpStream};
use std::os::unix::io::AsRawFd;
use std::ptr;
use std::ffi::CString;
use std::os::raw::c_char;

#[repr(C)]
pub struct RelaySession {
    sock: c_int,
    ssl: c_int,
    gnutls_session: *mut c_void,
}

impl RelaySession {
    pub fn new(addr: String) ->  *mut RelaySession {
        let stream = TcpStream::connect("127.0.0.1:8100").expect("Failed to connect to weechat server.");

        unsafe {
            weechat_relay_session_init(stream.as_raw_fd(), ptr::null_mut())
        }
    }

    pub fn init(&mut self, password: String) -> () {
        let c_password = CString::new(password).unwrap();
        unsafe {
            weechat_relay_cmd_init(&mut *self, c_password.as_ptr(), ptr::null_mut());
        }
    }
    fn shutdown(&self) {
        // TODO.
    }
}



#[allow(dead_code)]
#[link(name = "weechatrelay")]
extern "C" {
    // Create a new RelaySession object.
    pub fn weechat_relay_session_init(sock: c_int, gnutls_session: *mut c_void) -> *mut RelaySession;

    // Close the relay session.
    pub fn weechat_relay_session_free(gnutls_session: *mut RelaySession) -> ();

    // Relay Commands.
    pub fn weechat_relay_cmd_raw(gnutls_session: *mut RelaySession,
                                 buffer: *const u8,
                                 size: size_t) -> c_int;

    // pub fn weechat_relay_cmd(gnutls_session: *mut RelaySession,
    //                      msg_id: *const u8,
    //                      command: *const u8,
    //                      arguments: ) -> c_int;

    pub fn weechat_relay_cmd_init(gnutls_session: *const RelaySession,
                                  password: *const c_char,
                                  compression: *const c_char) -> c_int;


    pub fn weechat_relay_cmd_hdata(gnutls_session: *mut RelaySession,
                                   msg_id: *const u8,
                                   path: *const u8,
                                   keys: *const u8) -> c_int;

    pub fn weechat_relay_cmd_info(gnutls_session: *mut RelaySession,
                                  msg_id: *const i8,
                                  name: *const i8) -> c_int;

    pub fn weechat_relay_cmd_infolist(gnutls_session: *mut RelaySession,
                                      msg_id: *const u8,
                                      name: *const u8,
                                      pointer: *const u8,
                                      arguments: *const u8) -> c_int;

    pub fn weechat_relay_cmd_nicklist(gnutls_session: *mut RelaySession,
                                      msg_id: *const u8,
                                      buffer: *const u8) -> c_int;

    pub fn weechat_relay_cmd_input(gnutls_session: *mut RelaySession,
                                   buffer: *const u8,
                                   data: *const u8) -> c_int;

    pub fn weechat_relay_cmd_sync(gnutls_session: *mut RelaySession,
                                  buffers: *const u8,
                                  options: *const u8) -> c_int;

    pub fn weechat_relay_cmd_desync (gnutls_session: *mut RelaySession,
                                     buffers: *const u8,
                                     options: *const u8) -> c_int;

    pub fn weechat_relay_cmd_test (gnutls_session: *mut RelaySession) -> c_int;

    pub fn weechat_relay_cmd_ping (gnutls_session: *mut RelaySession,
                                   arguments: *const u8) -> c_int;

    pub fn weechat_relay_cmd_quit (gnutls_session: *mut RelaySession) -> c_int;
}
