use std::sync::mpsc::{Receiver, RecvError, SendError, SyncSender};

use crate::models::{cache::CacheState, processor::Processor, Data};

#[derive(Clone, Copy)]
pub struct BusSignal {
    pub origin: usize,
    pub address: usize,
    pub action: BusAction,
}

#[derive(Clone, Copy)]
pub enum BusAction {
    Invalidate,
    ReadMiss,
    WriteMem(Data),
}

pub struct Bus {
    cache_data_input: Receiver<Option<Data>>,
    signal_input: Receiver<BusSignal>,
    controllers: Vec<SyncSender<BusSignal>>,
    data_inputs: Vec<SyncSender<(CacheState, Data)>>,
}

impl Bus {
    pub fn new(
        bus_signal_receiver: Receiver<BusSignal>,
        bus_data_receiver: Receiver<Option<Data>>,
    ) -> Self {
        Self {
            signal_input: bus_signal_receiver,
            cache_data_input: bus_data_receiver,
            controllers: Vec::new(),
            data_inputs: Vec::new(),
        }
    }

    pub fn register_processor(&mut self, processor: &Processor) {
        self.controllers.push(processor.controller_signal_input());
        self.data_inputs.push(processor.cpu_data_input());
    }

    // This will panick on error but threads should just silently die i think
    pub fn recv_signal(&self) -> Result<BusSignal, RecvError> {
        self.signal_input.recv()
    }

    pub fn recv_data(&self) -> Result<Option<Data>, RecvError> {
        self.cache_data_input.recv()
    }

    pub fn propagate_signal(
        &self,
        signal: BusSignal,
    ) -> Result<(), SendError<BusSignal>> {
        for (i, sender) in self.controllers.iter().enumerate() {
            if i != signal.origin {
                sender.send(signal)?;
            }
        }
        Ok(())
    }

    pub fn check_cache_data(&self) -> Result<Option<Data>, RecvError> {
        let mut result = None;
        for _ in 0..self.controllers.len() - 1 {
            if let Some(data) = self.recv_data()? {
                result = Some(data);
            }
        }
        Ok(result)
    }

    pub fn send_data_to_cpu(
        &self,
        cpu: usize,
        status: CacheState,
        data: Data,
    ) -> Result<(), SendError<(CacheState, u16)>> {
        self.data_inputs[cpu].send((status, data))
    }
}
