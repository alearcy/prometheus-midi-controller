use crate::{faders::Faders, midi};
use crate::midi::MidiSettings;
use crate::serial::SerialSettings;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, WebSocketUpgrade, State,
    },
    headers,
    response::IntoResponse,
    routing::get,
    Router, TypedHeader,
};
use eframe::egui::{
    Button, CentralPanel, Color32, ComboBox, Context, Direction, Id, Layout, RichText, Sense,
    Slider, Ui,
};
use eframe::emath::Align;
use eframe::epaint::Vec2;
use eframe::NativeOptions;
use futures::stream::StreamExt;
use midi_control::*;
use midir::MidiOutputConnection;
use serialport::SerialPort;
use std::error::Error;
use std::io::BufRead;
use std::io::BufReader;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{collections::HashMap, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};
use local_ip_address::local_ip;
struct App {
    selected_midi_device: String,
    selected_serial_port: String,
    faders: Faders,
    available_midi_devices: HashMap<String, midir::MidiOutputPort>,
    available_serial_ports: HashMap<String, String>,
    f1value: Arc<Mutex<u8>>,
    f2value: Arc<Mutex<u8>>,
    f3value: Arc<Mutex<u8>>,
    f4value: Arc<Mutex<u8>>,
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
            f1value: Arc::new(Mutex::new(0)),
            f2value: Arc::new(Mutex::new(0)),
            f3value: Arc::new(Mutex::new(0)),
            f4value: Arc::new(Mutex::new(0)),
            is_connected: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let app_rect = ui.max_rect();
            let title_bar_height = 16.0;
            let title_bar_rect = {
                let mut rect = app_rect;
                rect.max.y = title_bar_height;
                rect
            };
            window_bar_ui(ui, frame, title_bar_rect);
            ui.add_enabled_ui(!self.is_connected, |ui| {
                ComboBox::from_label("MIDI out")
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
                ComboBox::from_label("Serial port")
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
            });

            let button_text = if self.is_connected {
                "Disconnect"
            } else {
                "Connect"
            };
            let (button_bg, _button_color) = if self.is_connected {
                (Color32::GREEN, Color32::BLACK)
            } else {
                (Color32::GRAY, Color32::BLACK)
            };
            if ui
                .add(
                    Button::new(button_text)
                        .fill(button_bg)
                        .min_size(Vec2 { x: 220.0, y: 32.0 }),
                )
                .clicked()
            {
                let (midi, mut serial, is_connected) = connect(
                    &self.available_midi_devices,
                    &self.selected_midi_device,
                    &self.selected_serial_port,
                    &self.available_serial_ports,
                )
                .unwrap();

                if is_connected {
                    self.is_connected = true;
                    let my_local_ip = local_ip();
                    if let Ok(my_local_ip) = my_local_ip {
                        println!("This is my local IP address: {:?}", my_local_ip);
                    } else {
                        println!("Error getting local IP: {:?}", my_local_ip);
                    }
                    let faders = self.faders.clone();
                    let f1value = Arc::clone(&self.f1value);
                    let f2value = Arc::clone(&self.f2value);
                    let f3value = Arc::clone(&self.f3value);
                    let f4value = Arc::clone(&self.f4value);
                    let context = ctx.clone();

                    let midi = Arc::new(Mutex::new(midi));
                    let midi_clone = Arc::clone(&midi);

                    tokio::spawn(async move {
                        let assets_dir =
                            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("app/build");
                        let server = Router::new()
                            .fallback_service(
                                ServeDir::new(assets_dir).append_index_html_on_directories(true),
                            )
                            .route("/ws", get(ws_message))
                            .layer(
                                TraceLayer::new_for_http().make_span_with(
                                    DefaultMakeSpan::default().include_headers(true),
                                ),
                            )
                            .with_state(midi_clone);
                        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
                        axum::Server::bind(&addr)
                            .serve(server.into_make_service_with_connect_info::<SocketAddr>())
                            .await
                            .unwrap();
                    });

                    thread::spawn(move || {
                        let midi_clone = Arc::clone(&midi);
                        let mut reader = BufReader::new(&mut *serial);
                        let mut my_str = String::new();
                        loop {
                            match reader.read_line(&mut my_str) {
                                Ok(_n) => {
                                    let string_to_split = my_str.clone();
                                    my_str.clear();
                                    let message_values: Vec<&str> =
                                        string_to_split.split(",").collect();
                                    let arduino_pin = message_values[0].parse().unwrap();
                                    let value = message_values[1].to_owned();
                                    let parsed_value: &u8 =
                                        &value[0..value.len() - 1].to_owned().parse().unwrap();
                                    let cc = faders.pins.get(&arduino_pin).unwrap();
                                    match arduino_pin {
                                        18 => {
                                            let mut f1 = f1value.lock().unwrap();
                                            *f1 = parsed_value.clone();
                                            context.request_repaint();
                                        }
                                        19 => {
                                            let mut f2 = f2value.lock().unwrap();
                                            *f2 = parsed_value.clone();
                                            context.request_repaint();
                                        }
                                        20 => {
                                            let mut f3 = f3value.lock().unwrap();
                                            *f3 = parsed_value.clone();
                                            context.request_repaint();
                                        }
                                        21 => {
                                            let mut f4 = f4value.lock().unwrap();
                                            *f4 = parsed_value.clone();
                                            context.request_repaint();
                                        }
                                        _ => (),
                                    }
                                    let message = midi_control::control_change(
                                        Channel::Ch1,
                                        *cc,
                                        *parsed_value,
                                    );
                                    let message_byte: Vec<u8> = message.into();
                                    let _ = &mut midi_clone.lock().unwrap().send(&message_byte).unwrap();
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
            ui.with_layout(
                Layout::centered_and_justified(Direction::LeftToRight),
                |ui| {
                    ui.add(
                        Slider::new(&mut *self.f1value.lock().unwrap(), 0..=127)
                            .text("CC1")
                            .vertical(),
                    );
                    ui.add(
                        Slider::new(&mut *self.f2value.lock().unwrap(), 0..=127)
                            .text("CC11")
                            .vertical(),
                    );
                    ui.add(
                        Slider::new(&mut *self.f3value.lock().unwrap(), 0..=127)
                            .text("CC2")
                            .vertical(),
                    );
                    ui.add(
                        Slider::new(&mut *self.f4value.lock().unwrap(), 0..=127)
                            .text("CC3")
                            .vertical(),
                    );
                },
            );
        });
    }
}

fn window_bar_ui(ui: &mut Ui, frame: &mut eframe::Frame, title_bar_rect: eframe::epaint::Rect) {
    let title_bar_response = ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());

    if title_bar_response.is_pointer_button_down_on() {
        frame.drag_window();
    }

    ui.allocate_ui_at_rect(title_bar_rect, |ui| {
        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.visuals_mut().button_frame = false;
            close_minimize(ui, frame);
        });
    });
}

// Show some close/minimize buttons for the native window.
fn close_minimize(ui: &mut Ui, frame: &mut eframe::Frame) {
    let button_height = 12.0;

    let close_response = ui
        .add(Button::new(RichText::new("❌").size(button_height)))
        .on_hover_text("Close the window");
    if close_response.clicked() {
        frame.close();
    }

    let minimized_response = ui
        .add(Button::new(RichText::new("🗕").size(button_height)))
        .on_hover_text("Minimize the window");
    if minimized_response.clicked() {
        frame.set_minimized(true);
    }
}

fn connect(
    midi_devices: &HashMap<String, midir::MidiOutputPort>,
    selected_midi_device: &String,
    selected_serial_port: &String,
    serial_ports: &HashMap<String, String>,
) -> Result<(MidiOutputConnection, Box<dyn SerialPort>, bool), Box<dyn Error>> {
    let midi = MidiSettings::new();
    let mut serial = SerialSettings::new();
    let midi_connection = midi
        .midi_out
        .connect(midi_devices.get(selected_midi_device).unwrap(), "port_name")?;
    let port = serial_ports.get(selected_serial_port).unwrap();
    let serial_port = serial.set_new_port(
        port.to_string(),
        115_200,
        1,
        serialport::FlowControl::Hardware,
    )?;
    Ok((midi_connection, serial_port, true))
}

pub async fn run() -> Result<(), Box<dyn Error>> {
    let faders: Faders = Faders::default();
    let mut midi_instance = MidiSettings::new();
    let mut serial_instance = SerialSettings::new();
    let available_midi_devices = midi_instance.get_out_ports();
    let serial_ports = serial_instance.get_ports();
    let native_options = NativeOptions {
        initial_window_size: Some(Vec2::new(260.0, 260.0)),
        max_window_size: Some(Vec2::new(260.0, 260.0)),
        resizable: false,
        centered: true,
        icon_data: Some(
            eframe::IconData::try_from_png_bytes(include_bytes!("../assets/icon.png")).unwrap(),
        ),
        decorated: false,
        ..Default::default()
    };
    let app = App::new(available_midi_devices, serial_ports, faders);
    let _ = eframe::run_native(
        "Composer controller",
        native_options,
        Box::new(|_cc| Box::new(app)),
    );

    Ok(())
}

async fn ws_message(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(midi): State<Arc<Mutex<MidiOutputConnection>>>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, midi))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr, midi: Arc<Mutex<MidiOutputConnection>>) {
    let (_sender, mut receiver) = socket.split();
    let _recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if process_message(msg, who, midi.clone()).unwrap().is_break() {
                break;
            }
        }
    });
}

fn process_message(msg: Message, who: SocketAddr, midi: Arc<Mutex<MidiOutputConnection>>) -> Result<ControlFlow<(), ()>, ()> {
    match msg {
        Message::Text(t) => {
            let message: midi::ClientMessage = serde_json::from_str(&t).unwrap();
            println!(">>> {who} sent converted str: {message:?}");
            if message.midi_type == "cc" {
                let midi_message = midi_control::control_change(
                    Channel::Ch1,
                    message.cc_value,
                    message.value as u8,
                );
                let midi_message_byte: Vec<u8> = midi_message.into();
                midi.lock().unwrap().send(&midi_message_byte).unwrap();
            }
            if message.midi_type == "note" {
                let midi_message = midi_control::note_on(
                    Channel::Ch1,
                    message.value as u8,
                    127,
                );
                let midi_message_byte: Vec<u8> = midi_message.into();
                midi.lock().unwrap().send(&midi_message_byte).unwrap();
            }
            if message.midi_type == "pitch" {
                let midi_message = midi_control::pitch_bend(
                    Channel::Ch1,
                    message.value as u16,
                );
                let midi_message_byte: Vec<u8> = midi_message.into();
                midi.lock().unwrap().send(&midi_message_byte).unwrap();
            }
        }
        Message::Binary(d) => {
            let message = String::from_utf8(d.clone()).unwrap();
            println!(
                ">>> {} sent {} bytes: {:?} message: {:?}",
                who,
                d.len(),
                d,
                message
            );
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow sent close message without CloseFrame");
            }
            return Ok(ControlFlow::Break(()));
        }
        Message::Pong(v) => {
            println!(">>> {who} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {who} sent ping with {v:?}");
        }
    }
    Ok(ControlFlow::Continue(()))
}
