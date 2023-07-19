use console::{Style, Term};
use dialoguer::{theme::ColorfulTheme, FuzzySelect, Input};
use midi_control::*;
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use std::collections::HashMap;
use std::io::BufReader;
use std::io::BufRead;
use std::io::ErrorKind::TimedOut;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

struct Faders {
    pins: HashMap<u8, u8>
}

impl Faders {
    fn default() -> Self {
        Self {
            pins: HashMap::from([(18, 1), (19, 11), (20, 2), (21, 3)])
        }
    }
}

fn main() {
    // (arduino pin, cc)
    let mut faders: Faders = Faders::default();

    let mut midi = midi_term_settings().unwrap();
    let mut serial = serial_term_settings().unwrap();
    let _ = faders_term_settings(&mut faders);
    run(&mut serial, &faders, &mut midi);
}

fn midi_term_settings() -> Option<MidiOutputConnection> {
    let red = Style::new().red();
    let cyan = Style::new().cyan();
    println!("Welcome to {}", red.apply_to("Prometheus"));
    let midi_out = MidiOutput::new("prometheus").unwrap();
    let ports = &midi_out.ports();
    let mut midi_devices_names: Vec<String> = Vec::new();
    let mut midi_ports: Vec<&MidiOutputPort> = Vec::new();
    if ports.len() > 0 {
        ports.iter().for_each(|p| {
            let midi_port_name = midi_out.port_name(p).unwrap();
            midi_devices_names.push(midi_port_name);
            midi_ports.push(p);
        });
    } else {
        println!("{}", cyan.apply_to("No MIDI output device found"));
    }

    println!("Select a {}", cyan.apply_to("MIDI output device"));

    let midi_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(&midi_devices_names)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap();
    let usable_port: &MidiOutputPort = if let Some(index) = midi_selection {
        let mut selected_midi_device_name: String = String::from("");
        selected_midi_device_name.push_str(midi_devices_names[index].as_str());
        midi_ports[index]
    } else {
        midi_ports[0]
    };
    Some(midi_out.connect(usable_port, "midir-forward").unwrap())
}

fn serial_term_settings() -> Option<Box<dyn SerialPort>> {
    let cyan = Style::new().cyan();
    let mut selected_port_name = String::from("");
    let mut serial_devices: Vec<String> = Vec::new();
    let serialports: Vec<SerialPortInfo> = serialport::available_ports().unwrap();
    let mut ports_info: HashMap<String, String> = HashMap::from([]);
    serialports.into_iter().for_each(|port| {
        let (visual_name, port_name) = match port.port_type {
            SerialPortType::UsbPort(info) => (info.product.unwrap(), port.port_name),
            SerialPortType::PciPort => (port.port_name.clone(), port.port_name),
            SerialPortType::BluetoothPort => (port.port_name.clone(), port.port_name),
            SerialPortType::Unknown => (port.port_name.clone(), port.port_name),
        };
        serial_devices.push(visual_name.clone());
        ports_info.insert(visual_name, port_name);
    });
    println!("Select a {}", cyan.apply_to("serial port"));
    let serial_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
        .items(&serial_devices)
        .default(0)
        .interact_on_opt(&Term::stderr())
        .unwrap();

    match serial_selection {
        Some(index) => {
            for (info, name) in ports_info.iter() {
                if &serial_devices[index] == info {
                    selected_port_name = name.to_string();
                }
            }
        },
        None => println!("You did not select anything"),
    }
    let mut port = serialport::new(&selected_port_name, 115_200)
        .timeout(Duration::from_millis(1))
        .open()
        .unwrap();
    port.set_flow_control(serialport::FlowControl::Hardware)
        .unwrap();

    Some(port)
}

fn faders_term_settings(faders: &mut Faders) {
    let red = Style::new().red();

    println!("The actual faders CC are {}", red.apply_to("1, 11, 2, 3"));

    let input: String = Input::new()
        .with_prompt(format!(
            "{}",
            String::from("Press [y] to accept")
        ))
        .interact_text()
        .unwrap();
    dbg!(&input);
    if input == "y" {
        return;
    } else {
        let faders_vec: Vec<&str> = input.split(",").collect();
        if faders_vec.len() == 4 {
            let first_value: u8 = faders_vec[0].to_owned().parse().unwrap();
            let second_value: u8 = faders_vec[1].to_owned().parse().unwrap();
            let third_value: u8 = faders_vec[2].to_owned().parse().unwrap();
            let fourth_value: u8 = faders_vec[3].to_owned().parse().unwrap();
            *faders.pins.get_mut(&18).unwrap() = first_value;
            *faders.pins.get_mut(&19).unwrap() = second_value;
            *faders.pins.get_mut(&20).unwrap() = third_value;
            *faders.pins.get_mut(&21).unwrap() = fourth_value;
        } else {
            println!("You have to insert at least 4 numbers");
        }
    }
}

fn run(
    serial: &mut Box<dyn SerialPort>,
    faders: &Faders,
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
    let red = Style::new().red();
    let input: String = Input::new()
        .with_prompt(format!(
            "{} {}",
            red.apply_to("Prometheus started!\n\n"),
            String::from("Press [q] to quit")
        ))
        .interact_text()
        .unwrap();
    input
}
