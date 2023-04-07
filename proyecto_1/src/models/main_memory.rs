use std::sync::mpsc::Sender;

use crate::app::Event;

pub struct Memory {
    blocks: usize,
    gui_tx: Sender<Event>,
}

impl Memory {
    pub fn new(blocks: usize, gui_sender: Sender<Event>) -> Memory {
        Memory {
            blocks,
            gui_tx: gui_sender,
        }
    }
}
