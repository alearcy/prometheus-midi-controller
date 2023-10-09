use eframe::egui::{Button, Slider};
use eframe::egui::{CentralPanel, ComboBox, Context};
use faders::Faders;
use midi_control::*;
use midir::MidiOutputConnection;
use serialport::SerialPort;
use std::collections::HashMap;
use std::error::Error;
use std::io::BufRead;
use std::io::BufReader;
use std::thread;
mod faders;
mod midi;
mod serial;

struct App {
    selected_midi_device: String,
    selected_serial_port: String,
    faders: Faders,
    available_midi_devices: HashMap<String, midir::MidiOutputPort>,
    available_serial_ports: HashMap<String, String>,
    f1value: u8,
    f2value: u8,
    f3value: u8,
    f4value: u8,
    is_connected: bool,
}

impl App {
    fn new(
        available_midi_devices: HashMap<String, midir::MidiOutputPort>,
        available_serial_ports: HashMap<String, String>,
        faders: Faders,
    ) -> Self {
        Self {
            available_midi_devices,
            available_serial_ports,
            selected_midi_device: String::from(""),
            selected_serial_port: String::from(""),
            faders,
            f1value: 0,
            f2value: 0,
            f3value: 0,
            f4value: 0,
            is_connected: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ComboBox::from_label("Choose MIDI output device")
                .selected_text(format!(
                    "{midi_device}",
                    midi_device = self.selected_midi_device
                ))
                .show_ui(ui, |ui| {
                    self.available_midi_devices
                        .clone()
                        .into_iter()
                        .for_each(|(name, _)| {
                            ui.selectable_value(
                                &mut self.selected_midi_device,
                                name.to_owned(),
                                name.to_owned(),
                            );
                        });
                });
            ComboBox::from_label("Choose serial port")
                .selected_text(format!(
                    "{serial_port}",
                    serial_port = self.selected_serial_port
                ))
                .show_ui(ui, |ui| {
                    self.available_serial_ports
                        .clone()
                        .into_iter()
                        .for_each(|(name, _)| {
                            ui.selectable_value(
                                &mut self.selected_serial_port,
                                name.to_owned(),
                                name.to_owned(),
                            );
                        });
                });
            if ui.add(Button::new("Connect")).clicked() {
                let (mut midi, mut serial, is_connected) = connect(
                    &self.available_midi_devices,
                    &self.selected_midi_device,
                    &self.selected_serial_port,
                    &self.available_serial_ports
                ).unwrap();
                if is_connected {
                    self.is_connected = true;
                    let faders = self.faders.clone();
                    thread::spawn(move || {
                        let mut reader = BufReader::new(&mut *serial);
                        let mut my_str = String::new();
                        loop {
                            match reader.read_line(&mut my_str) {
                                Ok(_n) => {
                                    let string_to_split = my_str.clone();
                                    my_str.clear();
                                    let message_values: Vec<&str> = string_to_split.split(",").collect();
                                    let arduino_pin = message_values[0].parse().unwrap();
                                    let value = message_values[1].to_owned();
                                    let parsed_value = &value[0..value.len() - 1].to_owned().parse().unwrap();
                                    let cc = faders.pins.get(&arduino_pin).unwrap();
                                    let message = midi_control::control_change(Channel::Ch1, *cc, *parsed_value);
                                    let message_byte: Vec<u8> = message.into();
                                    let _ = &mut midi.send(&message_byte).unwrap();
                                }
                                Err(error) => {
                                    if error.kind() == std::io::ErrorKind::TimedOut {
                                        continue;
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                    });
                }
            };
            ui.horizontal(|ui| {
                ui.add(
                    Slider::new(&mut self.f1value, 0..=127)
                        .text("CC1")
                        .vertical(),
                );
                ui.add(
                    Slider::new(&mut self.f2value, 0..=127)
                        .text("CC11")
                        .vertical(),
                );
                ui.add(
                    Slider::new(&mut self.f3value, 0..=127)
                        .text("CC2")
                        .vertical(),
                );
                ui.add(
                    Slider::new(&mut self.f4value, 0..=127)
                        .text("CC3")
                        .vertical(),
                );
            });
        });
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let faders: faders::Faders = faders::Faders::default();
    let available_midi_devices = set_midi_connection()?;
    let serial_ports = set_serial_connection();
    run(serial_ports, faders, available_midi_devices)?;
    Ok(())
}

fn set_midi_connection() -> Result<HashMap<String, midir::MidiOutputPort>, Box<dyn Error>> {
    let mut midi = midi::MidiSettings::new();
    let available_devices: HashMap<String, midir::MidiOutputPort> = midi.get_out_ports();
    Ok(available_devices)
}

fn set_serial_connection() -> HashMap<String, String> {
    let mut serial = serial::SerialSettings::new();
    let ports = serial.get_ports();
    ports
}

fn connect(
    midi_devices: &HashMap<String, midir::MidiOutputPort>,
    selected_midi_device: &String,
    selected_serial_port: &String,
    serial_ports: &HashMap<String, String>,
) -> Result<(MidiOutputConnection, Box<dyn SerialPort>, bool), Box<dyn Error>> {
    let midi_connection = midi::MidiSettings::new().midi_out.connect(
        midi_devices.get(selected_midi_device).unwrap(),
        "port_name",
    )?;
    let mut serial = serial::SerialSettings::new();
    let port = serial_ports.get(selected_serial_port).unwrap();
    serial.set_new_port(
        port.to_string(),
        115_200,
        1,
        serialport::FlowControl::Hardware,
    );
    Ok((midi_connection, serial.port, true))
}

fn run(
    serial: HashMap<String, String>,
    faders: faders::Faders,
    available_midi_devices: HashMap<String, midir::MidiOutputPort>,
) -> Result<(), std::io::Error> {
    let native_options = eframe::NativeOptions::default();
    let app = App::new(available_midi_devices, serial, faders);
    let _ = eframe::run_native(
        "AA - serial to midi",
        native_options,
        Box::new(|_cc| Box::new(app)),
    );
    Ok(())
}
