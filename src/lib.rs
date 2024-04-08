pub mod crypt;
pub mod tests;
pub mod utils;
pub mod net;
pub mod tui;
use tokio::sync::{mpsc, oneshot, OnceCell};
pub static INPUT_TX: OnceCell<mpsc::Sender<oneshot::Sender<String>>> = OnceCell::const_new();
