use midi_control::*;
use midir::{MidiOutputConnection};
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
mod faders;
mod midi;

fn main() {
    let faders: faders::Faders = faders::Faders::default();
    dbg!(faders);
    let mut midi_connection = set_midi_connection();
    let message = midi_control::control_change(Channel::Ch1, 1, 100);
    let message_byte: Vec<u8> = message.into();
    midi_connection.send(&message_byte).unwrap();
    // let mut serial = serial_term_settings().unwrap();
    // let _ = faders_term_settings(&mut faders);
    // run(&mut serial, &faders, &mut midi);
}

//TODO: return Option
fn set_midi_connection() -> MidiOutputConnection {
    let mut midi = midi::MidiSettings::new();
    let available_ports = midi.get_out_ports();
    let selected_midi_device = String::from("arcy-pc");
    return midi.midi_out.connect(&available_ports.get(&selected_midi_device).unwrap(), "port_name").unwrap();
}

// fn serial_term_settings() -> Option<Box<dyn SerialPort>> {
//     let cyan = Style::new().cyan();
//     let mut selected_port_name = String::from("");
//     let mut serial_devices: Vec<String> = Vec::new();
//     let serialports: Vec<SerialPortInfo> = serialport::available_ports().unwrap();
//     let mut ports_info: HashMap<String, String> = HashMap::from([]);
//     serialports.into_iter().for_each(|port| {
//         let (visual_name, port_name) = match port.port_type {
//             SerialPortType::UsbPort(info) => (info.product.unwrap(), port.port_name),
//             SerialPortType::PciPort => (port.port_name.clone(), port.port_name),
//             SerialPortType::BluetoothPort => (port.port_name.clone(), port.port_name),
//             SerialPortType::Unknown => (port.port_name.clone(), port.port_name),
//         };
//         serial_devices.push(visual_name.clone());
//         ports_info.insert(visual_name, port_name);
//     });
//     println!("Select a {}", cyan.apply_to("serial port"));
//     let serial_selection = FuzzySelect::with_theme(&ColorfulTheme::default())
//         .items(&serial_devices)
//         .default(0)
//         .interact_on_opt(&Term::stderr())
//         .unwrap();

//     match serial_selection {
//         Some(index) => {
//             for (info, name) in ports_info.iter() {
//                 if &serial_devices[index] == info {
//                     selected_port_name = name.to_string();
//                 }
//             }
//         },
//         None => println!("You did not select anything"),
//     }
//     let mut port = serialport::new(&selected_port_name, 115_200)
//         .timeout(Duration::from_millis(1))
//         .open()
//         .unwrap();
//     port.set_flow_control(serialport::FlowControl::Hardware)
//         .unwrap();

//     Some(port)
// }

// fn faders_term_settings(faders: &mut faders::Faders) {
//     let red = Style::new().red();

//     println!("The actual faders CC are {}", red.apply_to("1, 11, 2, 3"));

//     let input: String = Input::new()
//         .with_prompt(format!(
//             "{}",
//             String::from("Press [y] to accept")
//         ))
//         .interact_text()
//         .unwrap();
//     dbg!(&input);
//     if input == "y" {
//         return;
//     } else {
//         let faders_vec: Vec<&str> = input.split(",").collect();
//         if faders_vec.len() == 4 {
//             let first_value: u8 = faders_vec[0].to_owned().parse().unwrap();
//             let second_value: u8 = faders_vec[1].to_owned().parse().unwrap();
//             let third_value: u8 = faders_vec[2].to_owned().parse().unwrap();
//             let fourth_value: u8 = faders_vec[3].to_owned().parse().unwrap();
//             *faders.pins.get_mut(&18).unwrap() = first_value;
//             *faders.pins.get_mut(&19).unwrap() = second_value;
//             *faders.pins.get_mut(&20).unwrap() = third_value;
//             *faders.pins.get_mut(&21).unwrap() = fourth_value;
//         } else {
//             println!("You have to insert at least 4 numbers");
//         }
//     }
// }

// fn run(
//     serial: &mut Box<dyn SerialPort>,
//     faders: &faders::Faders,
//     midi: &mut MidiOutputConnection,
// ) {
//     let stop = Arc::new(AtomicBool::new(false));
//     let stop_me = stop.clone();
//     let thread = thread::spawn(move || {
//         let input = ask_for_quitting();
//         if input == "q" {
//             stop_me.store(true, Ordering::Relaxed);
//         }
//     });
//     let mut reader = BufReader::new(&mut *serial);
//     let mut my_str = String::new();
//     loop {
//         let read_line = reader.read_line(&mut my_str);
//         match read_line {
//             Ok(_n) => {
//                 let string_to_split = my_str.clone();
//                 my_str.clear();
//                 let message_values: Vec<&str> = string_to_split.split(",").collect();
//                 let arduino_pin = message_values[0].parse().unwrap();
//                 let value = message_values[1].to_owned();
//                 let parsed_value = &value[0..value.len() - 1].to_owned().parse().unwrap();
//                 let cc = faders.pins.get(&arduino_pin).unwrap();
//                 let message = midi_control::control_change(Channel::Ch1, *cc, *parsed_value);
//                 let message_byte: Vec<u8> = message.into();
//                 let _ = &mut midi.send(&message_byte).unwrap();
//             },
//             Err(e) => {
//                 if e.kind() == TimedOut {
//                     if stop.load(Ordering::Relaxed) == true {
//                         break;
//                     }
//                 }
//             }
//         }
//     }
//     thread.join().unwrap();
// }

// fn ask_for_quitting() -> String {
//     let red = Style::new().red();
//     let input: String = Input::new()
//         .with_prompt(format!(
//             "{} {}",
//             red.apply_to("Prometheus started!\n\n"),
//             String::from("Press [q] to quit")
//         ))
//         .interact_text()
//         .unwrap();
//     input
// }
