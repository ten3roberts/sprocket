#[derive(Debug)]
pub enum Event {
    WindowClose,
    WindowResize(i32, i32),
    MouseMove(i32, i32),
    Dummy(String),
}
