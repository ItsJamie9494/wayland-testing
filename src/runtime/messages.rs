// Messages from the runtime to the compositor
pub enum RuntimeMessage {
    Ping,
}

// Messages from the compositor to the runtime
pub enum CompositorMessage {
    Ping,
}
