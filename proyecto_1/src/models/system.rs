use std::sync::mpsc::{sync_channel, Sender};

use crate::{
    app::Event,
    models::{
        bus::Bus, cache::Cache, main_memory::Memory, processor::Processor,
    },
};

pub struct SocProperties {
    pub num_processors: usize,
    pub cache_associativity: usize,
    pub cache_sets: usize,
    pub main_memory_blocks: usize,
}

pub struct SoC {
    props: SocProperties,
    processors: Vec<Processor>,
    bus: Bus,
    main_memory: Memory,
}

impl SoC {
    pub fn init_system(props: SocProperties, gui_sender: Sender<Event>) -> SoC {
        let (bus_tx, bus_rx) = sync_channel(0);

        let mut processors = Vec::with_capacity(props.num_processors);
        let mut bus = Bus::new(bus_rx);
        let main_memory =
            Memory::new(props.main_memory_blocks, gui_sender.clone());

        for i in 0..props.num_processors {
            let mut cache =
                Cache::new_cold(i, props.cache_associativity, props.cache_sets);
            cache.register_gui_listener(gui_sender.clone());
            let processor =
                Processor::init(bus_tx.clone(), cache, gui_sender.clone());
            bus.register_processor(&processor);
            processors.push(processor);
        }

        SoC {
            props,
            processors,
            bus,
            main_memory,
        }
    }

    pub fn num_processors(&self) -> usize { self.props.num_processors }
    pub fn cache_associativity(&self) -> usize {
        self.props.cache_associativity
    }
    pub fn cache_sets(&self) -> usize { self.props.cache_sets }
    pub fn main_memory_size(&self) -> usize { self.props.main_memory_blocks }
}
