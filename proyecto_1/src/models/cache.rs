use std::sync::mpsc::Sender;

use crate::app::Event;

#[derive(Clone)]
pub struct Cache {
    gui_tx: Sender<Event>,
}

impl Cache {
    pub fn new(
        associativity: usize,
        sets: usize,
        gui_sender: Sender<Event>,
    ) -> Self {
        Self { gui_tx: gui_sender }
    }
}
