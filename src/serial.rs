use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use std::{collections::HashMap, time::Duration};

pub struct SerialSettings {
    pub serial_devices: Vec<String>,
    pub serialports: Vec<SerialPortInfo>,
    pub ports_info: HashMap<String, String>,
    pub port: Box<dyn SerialPort>,
}

impl SerialSettings {
    pub fn new() -> Self {
        Self {
            serial_devices: Vec::new(),
            serialports: serialport::available_ports().unwrap(),
            ports_info: HashMap::from([]),
            port: serialport::new("COM1", 115_200).open().unwrap(), //FIXME
        }
    }

    pub fn get_ports(&mut self) -> HashMap<String, String> {
        self.serialports.iter().for_each(|port| {
            let (visual_name, port_name) = match &port.port_type {
                SerialPortType::UsbPort(info) => (info.product.as_ref().unwrap(), &port.port_name),
                SerialPortType::PciPort => (&port.port_name, &port.port_name),
                SerialPortType::BluetoothPort => (&port.port_name, &port.port_name),
                SerialPortType::Unknown => (&port.port_name, &port.port_name),
            };
            self.ports_info
                .insert(visual_name.to_string(), port_name.to_string());
        });
        self.ports_info.clone()
    }

    pub fn set_new_port(
        &mut self,
        port: String,
        baud: u32,
        timeout: u64,
        flow_control: serialport::FlowControl,
    ) {
        let mut port = serialport::new(port, baud)
            .timeout(Duration::from_millis(timeout))
            .open()
            .unwrap();
        port.set_flow_control(flow_control).unwrap();
        self.port = port;
    }
}
