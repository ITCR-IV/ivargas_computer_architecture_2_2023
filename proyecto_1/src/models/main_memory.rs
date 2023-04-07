use std::sync::mpsc::Sender;

use crate::app::Event;

pub struct Memory {
    gui_tx: Sender<Event>,
}

impl Memory {
    pub fn new(gui_sender: Sender<Event>) -> Memory {
        Memory { gui_tx: gui_sender }
    }
}
