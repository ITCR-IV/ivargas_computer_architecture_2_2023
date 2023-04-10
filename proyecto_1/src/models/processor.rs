use std::{
    error::Error,
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
    models::{
        box_err,
        bus::BusSignal,
        cache::{Cache, CacheState},
        instructions::Instruction,
        Data,
    },
};

pub struct Processor {
    controller_signal_input: SyncSender<BusSignal>,
    cpu_instruction_input: SyncSender<Instruction>,
    cpu_data_input: SyncSender<(CacheState, Data)>,
}

fn controller_handle_signal(
    signal: BusSignal,
    cache: &mut Cache,
    bus_tx: &SyncSender<Option<Data>>,
) -> Result<(), Box<dyn Error>> {
    match signal.action {
        super::bus::BusAction::Invalidate => {
            cache.invalidate_address(signal.address);
            Ok(())
        }
        super::bus::BusAction::ReadMiss => {
            match cache.get_address_mut(signal.address) {
                Some(cache_line) => box_err(bus_tx.send(Some(cache_line.data))),
                None => box_err(bus_tx.send(None)),
            }
        }
        super::bus::BusAction::WriteMem(_) => todo!(),
    }
}

fn cpu_execute_instruction(
    instruction: Instruction,
    cache: &mut Cache,
    bus_tx: &SyncSender<BusSignal>,
    data_rx: &Receiver<(CacheState, Data)>,
) {
}

impl Processor {
    pub fn init(
        processor_i: usize,
        bus_signal_sender: SyncSender<BusSignal>,
        bus_data_sender: SyncSender<Option<Data>>,
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
                    bus_signal_sender,
                    gui_sender,
                )
            });
        }

        {
            let cache_lock = local_cache.clone();
            thread::spawn(move || {
                Self::controller_thread(
                    processor_i,
                    cache_lock,
                    controller_rx,
                    bus_data_sender,
                )
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

    pub fn cpu_data_input(&self) -> SyncSender<(CacheState, Data)> {
        self.cpu_data_input.clone()
    }

    pub fn controller_signal_input(&self) -> SyncSender<BusSignal> {
        self.controller_signal_input.clone()
    }

    fn cpu_thread(
        processor_i: usize,
        cache_lock: Arc<Mutex<Cache>>,
        instruction_rx: Receiver<Instruction>,
        data_rx: Receiver<(CacheState, Data)>,
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
        bus_tx: SyncSender<Option<Data>>,
    ) {
        let mut cache = cache_lock.lock().unwrap();

        loop {
            match controller_rx.try_recv() {
                Ok(signal) => {
                    if let Err(_) =
                        controller_handle_signal(signal, &mut cache, &bus_tx)
                    {
                        println!("Controller {processor_i} dying.");
                        break;
                    };
                } // Handle signal
                Err(TryRecvError::Disconnected) => break,
                Err(TryRecvError::Empty) => {
                    Mutex::unlock(cache);
                    match controller_rx.recv() {
                        Ok(signal) => {
                            cache = cache_lock.lock().unwrap();
                            if let Err(_) = controller_handle_signal(
                                signal, &mut cache, &bus_tx,
                            ) {
                                println!("Controller {processor_i} dying.");
                                break;
                            };
                        }
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
