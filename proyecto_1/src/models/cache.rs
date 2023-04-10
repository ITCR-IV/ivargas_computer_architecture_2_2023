use std::{mem::size_of, ops::Range, slice::SliceIndex, sync::mpsc::Sender};

use crate::{app::Event, models::Data};

#[derive(Clone, Copy)]
pub enum CacheState {
    Modified,
    Owned,
    Exclusive,
    Shared,
    Invalid,
}

impl CacheState {
    pub fn to_letter(&self) -> &str {
        match self {
            CacheState::Modified => "M",
            CacheState::Owned => "O",
            CacheState::Exclusive => "E",
            CacheState::Shared => "S",
            CacheState::Invalid => "I",
        }
    }
}

#[derive(Clone)]
pub struct CacheLine {
    pub state: CacheState,
    pub tag: usize,
    pub data: Data,
}

impl CacheLine {
    pub fn new_cold() -> Self {
        Self {
            state: CacheState::Invalid,
            tag: 0,
            data: 0,
        }
    }
}

#[derive(Clone)]
pub struct Cache {
    processor_id: usize,
    associativity: usize,
    sets: usize,
    offset_bits: usize,
    offset_mask: usize,
    index_bits: usize,
    index_mask: usize,
    storage: Vec<CacheLine>,
    gui_tx: Option<Sender<Event>>,
}

impl Cache {
    pub fn new_cold(
        processor_id: usize,
        associativity: usize,
        sets: usize,
    ) -> Self {
        let mut index_bits = 0;
        let mut x = sets - 1;
        while x != 0 {
            x >>= 1;
            index_bits += 1;
        }

        let mut offset_bits = 0;
        let mut x = size_of::<Data>() - 1;
        while x != 0 {
            x >>= 1;
            offset_bits += 1;
        }

        // u16: 0x0000_0003
        let offset_mask = !(((!0) >> offset_bits) << offset_bits);

        // 2 sets: 0x0000_0004
        let index_mask = (!(((!0) >> index_bits) << index_bits)) << offset_bits;

        Self {
            processor_id,
            offset_bits,
            offset_mask,
            index_bits,
            index_mask,
            associativity,
            sets,
            storage: vec![CacheLine::new_cold(); sets * associativity],
            gui_tx: None,
        }
    }

    pub fn register_gui_listener(&mut self, gui_tx: Sender<Event>) {
        self.gui_tx = Some(gui_tx);
    }

    pub fn associativity(&self) -> usize { self.associativity }

    pub fn sets(&self) -> usize { self.sets }

    pub fn blocks(&self) -> usize { self.sets * self.associativity }

    pub fn get_storage<I: SliceIndex<[CacheLine]>>(
        &self,
        index: I,
    ) -> Option<&<I as SliceIndex<[CacheLine]>>::Output> {
        self.storage.get(index)
    }

    pub fn get_tag(&self, address: usize) -> usize {
        (address & !(self.index_mask | self.offset_mask))
            >> (self.offset_bits + self.index_bits)
    }

    pub fn get_index(&self, address: usize) -> usize {
        (address & self.index_mask) >> self.offset_bits
    }

    fn get_offset(&self, address: usize) -> usize { address & self.offset_mask }

    pub fn get_set(&self, index: usize) -> Option<&[CacheLine]> {
        let set_range =
            index * self.associativity..(index + 1) * self.associativity;
        self.get_storage(set_range)
    }

    fn get_set_range(&self, index: usize) -> Range<usize> {
        index * self.associativity..(index + 1) * self.associativity
    }

    pub fn get_set_mut(&mut self, index: usize) -> Option<&mut [CacheLine]> {
        let set_range =
            index * self.associativity..(index + 1) * self.associativity;
        self.storage.get_mut(set_range)
    }

    pub fn store_line(&mut self, block_index: usize, line: CacheLine) {
        if let Some(ref sender) = self.gui_tx {
            sender
                .send(Event::CacheWrite {
                    cache_i: self.processor_id,
                    block_i: block_index,
                    line: line.clone(),
                })
                .ok();
        }
        self.storage[block_index] = line;
    }

    pub fn store_address(&mut self, address: usize, line: CacheLine) {
        self.store_line(address << self.offset_bits, line);
    }

    pub fn invalidate_address(&mut self, address: usize) {
        let index = self.get_index(address);

        for i in self.get_set_range(index) {
            if self.storage[i].tag == self.get_tag(address) {
                let mut cache_line = &mut self.storage[i];
                cache_line.state = CacheState::Invalid;
            }
        }
    }
}
