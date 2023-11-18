//! Takes an audio input and output it to an audio output.
//! Makes a copy of the audio in a file
//! Three commandline arguments:
//! 1. In port name
//! 2. Out port name
//! 3. String holing path for output file
//! All JACK notifications are also printed out.

use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::{self};
fn main() {
    // let mut args = env::args();
    // let file_name = args.next().expect("Pass the file name prefix");
    // let _file = File::create(file_name.as_str()).expect("Opening file {file_name}");

    // Create client
    let (client, _status) =
        jack::Client::new("jackrec_qzt", jack::ClientOptions::NO_START_SERVER).unwrap();
    // The `in_ports` that match "system:playback" are the audio output
    let in_ports = client.ports(Some("system:playback"), None, jack::PortFlags::IS_INPUT);
    let out_ports = client.ports(None, None, jack::PortFlags::IS_OUTPUT);
    let mut ports: Vec<String> = vec![];
    for p in out_ports.iter() {
        let outport = client.port_by_name(p.as_str()).unwrap();

        for name in in_ports.iter() {
            if outport.is_connected_to(name.as_str()).unwrap() {
                eprintln!("inp: {name} outp: {p}");
                ports.push(p.clone());
            }
        }
    }

    let mut clients = vec![];
    for name in ports.iter() {
        let (client, _status) =
            jack::Client::new("qzt", jack::ClientOptions::NO_START_SERVER).expect("Client qzt");
        let spec = jack::AudioIn;
        let inport = client.register_port(name, spec).unwrap();
        let to_port = inport.name().as_ref().unwrap().to_string();
        let fname = format!("{name}.raw");
        let file = File::create(fname.as_str()).expect("Opening file {name}");
        let mut writer = BufWriter::new(file);
        let process_callback =
            move |_jc: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
                let in_a_p: &[f32] = inport.as_slice(ps);
                for v in in_a_p {
                    let bytes = v.to_ne_bytes();
                    writer.write_all(&bytes).unwrap();
                }
                writer.flush().unwrap();
                jack::Control::Continue
            };

        let process = jack::ClosureProcessHandler::new(process_callback);
        // Activate the client, which starts the processing.
        let _active_client = client.activate_async(Notifications, process).unwrap();
        let from_port = name;

        let (client, _status) =
            jack::Client::new("qzt", jack::ClientOptions::NO_START_SERVER).expect("Client qzt");
        match client.connect_ports_by_name(from_port.as_str(), to_port.as_str()) {
            //inport.name().unwrap
            Ok(()) => eprintln!("Registering {name} -> {}", to_port),
            Err(err) => {
                eprintln!("Failed  {name} -> {} '{err}'", to_port);
            }
        };
        clients.push(_active_client);
    }
    let mut input = String::new();

    io::stdin().read_line(&mut input).unwrap();
    for client in clients {
        client.deactivate().unwrap();
    }
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {srate}");
        jack::Control::Continue
    }
}
