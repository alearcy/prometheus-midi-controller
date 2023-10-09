use midir::{MidiOutput, MidiOutputPort};
use std::collections::HashMap;

pub struct MidiSettings {
    pub midi_out: MidiOutput,
    pub midi_device_names: Vec<String>,
    pub midi_ports: HashMap<String, MidiOutputPort>,
}

impl MidiSettings {
    pub fn new() -> Self {
        Self {
            midi_out: MidiOutput::new("prometheus").unwrap(),
            midi_device_names: Vec::new(),
            midi_ports: HashMap::new(),
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
