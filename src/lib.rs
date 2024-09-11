use tokio::sync::broadcast;

pub mod cf;
pub mod cli;
pub mod ip;
pub mod worker;

#[derive(Clone, Debug)]
pub struct Message {
    ip: String,
}

pub type MessageSender = broadcast::Sender<Message>;
pub type MessageReceiver = broadcast::Receiver<Message>;
pub type ShutdownSender = broadcast::Sender<bool>;
pub type ShutdownReceiver = broadcast::Receiver<bool>;
