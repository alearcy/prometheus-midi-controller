use midir::{MidiOutput, MidiOutputPort};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub struct MidiSettings {
    pub midi_out: MidiOutput,
    pub midi_device_names: Vec<String>,
    pub midi_ports: HashMap<String, MidiOutputPort>,
    pub selected_midi_device: String,
}

impl MidiSettings {
    pub fn new() -> Self {
        Self {
            midi_out: MidiOutput::new("prometheus").unwrap(),
            midi_device_names: Vec::new(),
            midi_ports: HashMap::new(),
            selected_midi_device: String::from(""),
        }
    }

    pub fn get_out_ports(&mut self) -> HashMap<String, MidiOutputPort> {
        let ports: Vec<MidiOutputPort> = self.midi_out.ports();
        if ports.len() > 0 {
            ports.iter().for_each(|p| {
                let midi_port_name = self.midi_out.port_name(p).unwrap();
                self.midi_ports.insert(midi_port_name, p.clone());
            });
        }
        self.midi_ports.clone()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientMessage {
    pub midi_type: String,
    pub channel: u8,
    pub cc_value: u8,
    pub value: u16,
}