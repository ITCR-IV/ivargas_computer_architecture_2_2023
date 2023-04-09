use std::{mem::size_of, slice::SliceIndex, sync::mpsc::Sender};

use crate::{app::Event, models::Data};

pub struct Memory {
    blocks: usize,
    storage: Vec<Data>,
    gui_tx: Option<Sender<Event>>,
    offset_bits: usize,
}

impl Memory {
    pub fn new(blocks: usize) -> Memory {
        let mut offset_bits = 0;
        let mut x = size_of::<Data>() - 1;
        while x != 0 {
            x >>= 1;
            offset_bits += 1;
        }
        Memory {
            offset_bits,
            blocks,
            storage: vec![0; blocks],
            gui_tx: None,
        }
    }

    pub fn register_gui_listener(&mut self, gui_tx: Sender<Event>) {
        self.gui_tx = Some(gui_tx);
    }

    pub fn get_line(&self, address: usize) -> usize {
        address >> self.offset_bits
    }

    pub fn get_storage<I: SliceIndex<[Data]>>(
        &self,
        index: I,
    ) -> Option<&<I as SliceIndex<[Data]>>::Output> {
        self.storage.get(index)
    }

    pub fn store_line(&mut self, block_index: usize, data: Data) {
        if let Some(ref sender) = self.gui_tx {
            sender
                .send(Event::MemWrite {
                    block_i: block_index,
                    data,
                })
                .ok();
        }
        self.storage[block_index] = data;
    }
}
