use std::{
    sync::mpsc::{sync_channel, RecvError, Sender, SyncSender},
    thread,
};

use crate::{
    app::Event,
    models::{
        bus::Bus, cache::Cache, instructions::Instruction, main_memory::Memory,
        processor::Processor,
    },
};

pub struct SocProperties {
    pub num_processors: usize,
    pub cache_associativity: usize,
    pub cache_sets: usize,
    pub main_memory_blocks: usize,
}

pub fn init_system(
    props: SocProperties,
    gui_sender: Sender<Event>,
) -> Vec<SyncSender<Instruction>> {
    let (bus_tx, bus_rx) = sync_channel(0);

    let mut processors = Vec::with_capacity(props.num_processors);
    let mut bus = Bus::new(bus_rx);
    let mut main_memory = Memory::new(props.main_memory_blocks);
    main_memory.register_gui_listener(gui_sender.clone());

    for i in 0..props.num_processors {
        let mut cache =
            Cache::new_cold(i, props.cache_associativity, props.cache_sets);
        cache.register_gui_listener(gui_sender.clone());
        let processor =
            Processor::init(i, bus_tx.clone(), cache, gui_sender.clone());
        bus.register_processor(&processor);
        processors.push(processor);
    }

    thread::spawn(move || system_control_thread(props, bus, main_memory));

    processors
        .iter()
        .map(|p| p.cpu_instruction_input())
        .collect()
}

fn system_control_thread(props: SocProperties, bus: Bus, main_memory: Memory) {
    loop {
        match bus.recv_signal() {
            Ok(signal) => {
                todo!()
            }
            Err(RecvError) => {
                println!("Bus dying.");
                break;
            }
        }
    }
}
