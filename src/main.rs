use dialoguer::Input;
use midi_control::*;
use midir::MidiOutputConnection;
use serialport::SerialPort;
use std::io::BufReader;
use std::io::BufRead;
use std::io::ErrorKind::TimedOut;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
mod faders;
mod midi;
mod serial;

fn main() {
    let faders: faders::Faders = faders::Faders::default();
    dbg!(&faders);
    let mut midi_connection = set_midi_connection();
    let message = midi_control::control_change(Channel::Ch1, 1, 100);
    let message_byte: Vec<u8> = message.into();
    midi_connection.send(&message_byte).unwrap();
    let mut serial = set_serial_connection();
    run(&mut serial, &faders, &mut midi_connection);
}

//TODO: return Option
fn set_midi_connection() -> MidiOutputConnection {
    let mut midi = midi::MidiSettings::new();
    let available_ports = midi.get_out_ports();
    let selected_midi_device = String::from("arcy-pc");
    return midi.midi_out.connect(&available_ports.get(&selected_midi_device).unwrap(), "port_name").unwrap();
}

//TODO: return Option
fn set_serial_connection() -> Box<dyn SerialPort> {
    let mut serial = serial::SerialSettings::new();
    let available_ports = serial.get_ports();
    dbg!(available_ports);
    serial.set_new_port("COM13".to_owned(), 115_200, 1, serialport::FlowControl::Hardware);
    serial.port
}

fn run(
    serial: &mut Box<dyn SerialPort>,
    faders: &faders::Faders,
    midi: &mut MidiOutputConnection,
) {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_me = stop.clone();
    let thread = thread::spawn(move || {
        let input = ask_for_quitting();
        if input == "q" {
            stop_me.store(true, Ordering::Relaxed);
        }
    });
    let mut reader = BufReader::new(&mut *serial);
    let mut my_str = String::new();
    loop {
        let read_line = reader.read_line(&mut my_str);
        match read_line {
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
            },
            Err(e) => {
                if e.kind() == TimedOut {
                    if stop.load(Ordering::Relaxed) == true {
                        break;
                    }
                }
            }
        }
    }
    thread.join().unwrap();
}

fn ask_for_quitting() -> String {
    let input: String = Input::new()
        .with_prompt(format!(
            "{} {}",
            "Prometheus started!\n\n",
            String::from("Press [q] to quit")
        ))
        .interact_text()
        .unwrap();
    input
}

// change faders
//             let first_value: u8 = faders_vec[0].to_owned().parse().unwrap();
//             let second_value: u8 = faders_vec[1].to_owned().parse().unwrap();
//             let third_value: u8 = faders_vec[2].to_owned().parse().unwrap();
//             let fourth_value: u8 = faders_vec[3].to_owned().parse().unwrap();
//             *faders.pins.get_mut(&18).unwrap() = first_value;
//             *faders.pins.get_mut(&19).unwrap() = second_value;
//             *faders.pins.get_mut(&20).unwrap() = third_value;
//             *faders.pins.get_mut(&21).unwrap() = fourth_value;