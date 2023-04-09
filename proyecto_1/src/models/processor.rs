use std::{
    sync::{
        mpsc::{
            sync_channel, Receiver, RecvError, Sender, SyncSender, TryRecvError,
        },
        Arc, Mutex,
    },
    thread,
};

use crate::{
    app::Event,
    models::{bus::BusSignal, cache::Cache, instructions::Instruction, Data},
};

pub struct Processor {
    controller_signal_input: SyncSender<BusSignal>,
    cpu_instruction_input: SyncSender<Instruction>,
    cpu_data_input: SyncSender<Data>,
}

fn controller_handle_signal(signal: BusSignal, cache: &mut Cache) {}

fn cpu_execute_instruction(
    instruction: Instruction,
    cache: &mut Cache,
    bus_tx: &SyncSender<BusSignal>,
    data_rx: &Receiver<Data>,
) {
}

impl Processor {
    pub fn init(
        processor_i: usize,
        bus_sender: SyncSender<BusSignal>,
        cache: Cache,
        gui_sender: Sender<Event>,
    ) -> Processor {
        let (cpu_instruction_tx, cpu_instruction_rx) = sync_channel(1);
        let (cpu_data_tx, cpu_data_rx) = sync_channel(0);
        let (controller_tx, controller_rx) = sync_channel(0);

        let local_cache = Arc::new(Mutex::new(cache));

        {
            let cache_lock = local_cache.clone();
            thread::spawn(move || {
                Self::cpu_thread(
                    processor_i,
                    cache_lock,
                    cpu_instruction_rx,
                    cpu_data_rx,
                    bus_sender.clone(),
                    gui_sender.clone(),
                )
            });
        }

        {
            let cache_lock = local_cache.clone();
            thread::spawn(move || {
                Self::controller_thread(processor_i, cache_lock, controller_rx)
            });
        }

        Processor {
            cpu_data_input: cpu_data_tx,
            cpu_instruction_input: cpu_instruction_tx,
            controller_signal_input: controller_tx,
        }
    }

    pub fn cpu_instruction_input(&self) -> SyncSender<Instruction> {
        self.cpu_instruction_input.clone()
    }

    pub fn cpu_data_input(&self) -> SyncSender<Data> {
        self.cpu_data_input.clone()
    }

    pub fn controller_signal_input(&self) -> SyncSender<BusSignal> {
        self.controller_signal_input.clone()
    }

    fn cpu_thread(
        processor_i: usize,
        cache_lock: Arc<Mutex<Cache>>,
        instruction_rx: Receiver<Instruction>,
        data_rx: Receiver<Data>,
        bus_tx: SyncSender<BusSignal>,
        gui_sender: Sender<Event>,
    ) {
        loop {
            match instruction_rx.recv() {
                Ok(instruction) => {
                    let mut cache = cache_lock.lock().unwrap();
                    cpu_execute_instruction(
                        instruction,
                        &mut cache,
                        &bus_tx,
                        &data_rx,
                    )
                }
                Err(RecvError) => {
                    println!("Processor {processor_i} dying.");
                    break;
                }
            }
        }
    }

    fn controller_thread(
        processor_i: usize,
        cache_lock: Arc<Mutex<Cache>>,
        controller_rx: Receiver<BusSignal>,
    ) {
        let mut cache = cache_lock.lock().unwrap();

        loop {
            match controller_rx.try_recv() {
                Ok(signal) => controller_handle_signal(signal, &mut cache), // Handle signal
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {
                    Mutex::unlock(cache);
                    match controller_rx.recv() {
                        Ok(signal) => {
                            cache = cache_lock.lock().unwrap();
                            controller_handle_signal(signal, &mut cache)
                        }
                        //Err(RecvError) => break,
                        Err(RecvError) => {
                            println!("Controller {processor_i} dying.");
                            break;
                        }
                    }
                }
            }
        }
    }
}
