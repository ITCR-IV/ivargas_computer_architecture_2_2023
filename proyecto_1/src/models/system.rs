use std::{
    error::Error,
    sync::mpsc::{sync_channel, RecvError, Sender, SyncSender},
    thread,
};

use crate::{
    app::Event,
    models::{
        bus::{Bus, BusAction, BusSignal},
        cache::{Cache, CacheState},
        instructions::Instruction,
        main_memory::Memory,
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
    let (bus_signal_tx, bus_signal_rx) = sync_channel(0);
    let (bus_data_tx, bus_data_rx) = sync_channel(0);

    let mut processors = Vec::with_capacity(props.num_processors);
    let mut bus = Bus::new(bus_signal_rx, bus_data_rx);
    let mut main_memory = Memory::new(props.main_memory_blocks);
    main_memory.register_gui_listener(gui_sender.clone());

    for i in 0..props.num_processors {
        let mut cache =
            Cache::new_cold(i, props.cache_associativity, props.cache_sets);
        cache.register_gui_listener(gui_sender.clone());
        let processor = Processor::init(
            i,
            bus_signal_tx.clone(),
            bus_data_tx.clone(),
            cache,
            gui_sender.clone(),
        );
        bus.register_processor(&processor);
        processors.push(processor);
    }

    thread::spawn(move || system_control_thread(props, bus, main_memory));

    processors
        .iter()
        .map(|p| p.cpu_instruction_input())
        .collect()
}

fn box_err<'a, E: Error + 'a>(
    res: Result<(), E>,
) -> Result<(), Box<dyn Error + 'a>> {
    match res {
        Ok(()) => Ok(()),
        Err(err) => Err(Box::new(err)),
    }
}

// This is called after the signal has already been propagated
fn handle_signal(
    signal: BusSignal,
    bus: &Bus,
    main_memory: &mut Memory,
) -> Result<(), Box<dyn Error>> {
    box_err(bus.propagate_signal(signal))?;

    match signal.action {
        BusAction::Invalidate => (),
        BusAction::ReadMiss => {
            return box_err(match bus.check_cache_data()? {
                Some(data) => bus.send_data_to_cpu(
                    signal.origin,
                    CacheState::Shared,
                    data,
                ),
                None => bus.send_data_to_cpu(
                    signal.origin,
                    CacheState::Exclusive,
                    main_memory.get_address(signal.address),
                ),
            });
        }
        BusAction::WriteMem(data) => {
            main_memory.store_address(signal.address, data)
        }
    }

    Ok(())
}

fn system_control_thread(
    props: SocProperties,
    bus: Bus,
    mut main_memory: Memory,
) {
    loop {
        match bus.recv_signal() {
            Ok(signal) => {
                if let Err(_) = handle_signal(signal, &bus, &mut main_memory) {
                    println!("Bus dying.");
                    break;
                };
            }
            Err(RecvError) => {
                println!("Bus dying.");
                break;
            }
        }
    }
}
