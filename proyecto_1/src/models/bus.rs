use std::sync::mpsc::{Receiver, SyncSender};

use crate::models::{processor::Processor, Data};

pub enum BusSignal {}
pub struct Bus {
    bus_input: Receiver<BusSignal>,
    controllers: Vec<SyncSender<BusSignal>>,
    data_inputs: Vec<SyncSender<Data>>,
}

impl Bus {
    pub fn new(bus_receiver: Receiver<BusSignal>) -> Self {
        Self {
            bus_input: bus_receiver,
            controllers: Vec::new(),
            data_inputs: Vec::new(),
        }
    }

    pub fn register_processor(&mut self, processor: &Processor) {
        self.controllers.push(processor.controller_signal_input());
        self.data_inputs.push(processor.cpu_data_input());
    }

    pub fn recv_signal(&self) -> BusSignal { self.bus_input.recv().unwrap() }
}
