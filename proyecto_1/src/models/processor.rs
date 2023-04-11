use std::{
    error::Error,
    sync::{
        mpsc::{sync_channel, Receiver, RecvError, Sender, SyncSender},
        Arc, Mutex,
    },
    thread,
};

use crate::{
    app::Event,
    models::{
        box_err,
        bus::{BusAction, BusSignal},
        cache::{Cache, CacheLine, CacheState},
        instructions::Instruction,
        Data, MemOp,
    },
};

pub struct Processor {
    controller_signal_input: SyncSender<BusSignal>,
    cpu_instruction_input: SyncSender<Instruction>,
    cpu_data_input: SyncSender<(CacheState, Data)>,
}

fn controller_handle_signal(
    processor_i: usize,
    signal: BusSignal,
    cache_lock: &Arc<Mutex<Cache>>,
    bus_tx: &SyncSender<Option<Data>>,
) -> Result<(), Box<dyn Error>> {
    let mut cache = cache_lock.lock().unwrap();
    match signal.action {
        BusAction::Invalidate => {
            println!(
                "({processor_i}) Controller invalidate {0}",
                signal.address
            );
            cache.invalidate_address(signal.address);

            Ok(())
        }
        BusAction::ReadMiss => {
            match cache.get_address(signal.address) {
                Some(cache_line) => {
                    println!(
                        "({processor_i}) Controller share {0}",
                        signal.address
                    );
                    let data = cache_line.data;
                    let state = cache_line.state;
                    Mutex::unlock(cache);
                    box_err(bus_tx.send(Some(data)))?;

                    // update the line's state
                    match state {
                        CacheState::Invalid => unreachable!(),
                        CacheState::Shared | CacheState::Owned => (),
                        CacheState::Exclusive | CacheState::Modified => {
                            let mut cache = cache_lock.lock().unwrap();
                            cache.change_state_address(
                                signal.address,
                                CacheState::Owned,
                            )
                        }
                    }
                    Ok(())
                }
                None => {
                    box_err(bus_tx.send(None))?;
                    Ok(())
                }
            }
        }
        BusAction::WriteMem(_) => Ok(()),
    }
}

fn maybe_write_back(
    address: usize,
    replaced_line: CacheLine,
    bus_tx: &SyncSender<BusSignal>,
    processor_i: usize,
) -> Result<(), Box<dyn Error>> {
    match replaced_line.state {
        CacheState::Invalid | CacheState::Shared | CacheState::Exclusive => (),
        // Write back
        // No need to invalidate because for Modified everything else should be invalid and for Owned the others can keep their copies
        CacheState::Modified | CacheState::Owned => {
            println!("({processor_i}) Asking to write back {address}");
            box_err(bus_tx.send(BusSignal {
                origin: processor_i,
                address,
                action: BusAction::WriteMem(replaced_line.data),
            }))?;
        }
    }

    Ok(())
}

fn cpu_execute_instruction(
    instruction: Instruction,
    cache_lock: &Arc<Mutex<Cache>>,
    bus_tx: &SyncSender<BusSignal>,
    data_rx: &Receiver<(CacheState, Data)>,
    gui_tx: &Sender<Event>,
    processor_i: usize,
) -> Result<(), Box<dyn Error>> {
    let mut cache = cache_lock.lock().unwrap();
    match instruction {
        Instruction::Calc => Ok(()),
        Instruction::Read { address } => match cache.get_address(address) {
            Some(_) => Ok(()),
            None => {
                gui_tx.send(Event::Alert {
                    address,
                    processor_i,
                    op: MemOp::Read,
                })?;
                Mutex::unlock(cache);

                box_err(bus_tx.send(BusSignal {
                    origin: processor_i,
                    address,
                    action: BusAction::ReadMiss,
                }))?;

                match data_rx.recv() {
                    Ok((state, data)) => {
                        let mut cache = cache_lock.lock().unwrap();
                        let replaced_line =
                            cache.store_line(address, state, data);

                        Mutex::unlock(cache);
                        maybe_write_back(
                            address,
                            replaced_line,
                            bus_tx,
                            processor_i,
                        )?;
                        Ok(())
                    }
                    Err(err) => Err(Box::new(err)),
                }
            }
        },

        Instruction::Write { address, data } => {
            let replaced_line =
                cache.store_line(address, CacheState::Modified, data);

            // report write miss
            if (replaced_line.tag != cache.get_tag(address))
                || (replaced_line.state == CacheState::Invalid)
            {
                gui_tx.send(Event::Alert {
                    address,
                    processor_i,
                    op: MemOp::Write,
                })?;
            }

            // invalidate other caches
            if replaced_line.state != CacheState::Modified {
                Mutex::unlock(cache);
                box_err(bus_tx.send(BusSignal {
                    origin: processor_i,
                    address,
                    action: BusAction::Invalidate,
                }))?
            } else {
                Mutex::unlock(cache);
            }

            maybe_write_back(address, replaced_line, bus_tx, processor_i)
        }
    }
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
                    if let Err(_) = cpu_execute_instruction(
                        instruction,
                        &cache_lock,
                        &bus_tx,
                        &data_rx,
                        &gui_sender,
                        processor_i,
                    ) {
                        println!("Processor {processor_i} dying.");
                        break;
                    }
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
        loop {
            match controller_rx.recv() {
                Ok(signal) => {
                    if let Err(_) = controller_handle_signal(
                        processor_i,
                        signal,
                        &cache_lock,
                        &bus_tx,
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
