use std::sync::OnceLock;

pub static PROTECT_SOCKET_CALLBACK: OnceLock<Box<dyn Fn(i32) -> bool + Send + Sync>> = OnceLock::new();

pub fn set_socket_protect_callback(cb: Box<dyn Fn(i32) -> bool + Send + Sync>) {
    let _ = PROTECT_SOCKET_CALLBACK.set(cb);
}

pub fn protect_socket(fd: i32) -> bool {
    if let Some(cb) = PROTECT_SOCKET_CALLBACK.get() {
        cb(fd)
    } else {
        false
    }
}
