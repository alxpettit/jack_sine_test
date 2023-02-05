//! Takes 2 audio inputs and outputs them to 2 audio outputs.
//! All JACK notifications are also printed out.
use jack::NotificationHandler;
use std::f32::consts::PI;
use std::io;
use std::time::Instant;

#[inline]
fn generate_sine_wave(
    frequency: f32,
    samples_per_sec: f32,
    buffer: &mut [f32],
    sample_count: &mut usize,
) {
    for x in buffer.iter_mut() {
        // Since frequency is measured in terms of seconds,
        // we must convert samples into seconds accordingly
        let time = (*sample_count as f32) / samples_per_sec;
        let v = 2.0 * std::f32::consts::PI * frequency * time;
        *x = (v.sin()) as f32;
        *sample_count += 1;
    }
}

fn main() {
    // Create client
    let (client, _status) =
        jack::Client::new("rust_jack_simple", jack::ClientOptions::NO_START_SERVER).unwrap();

    let mut out_a = client
        .register_port("rust_out_l", jack::AudioOut::default())
        .unwrap();
    let mut out_b = client
        .register_port("rust_out_r", jack::AudioOut::default())
        .unwrap();
    let mut sample_count: usize = 0;
    let process_callback = move |c: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let mut out_a_p = out_a.as_mut_slice(ps);
        let mut out_b_p = out_b.as_mut_slice(ps);

        generate_sine_wave(200.0, 48_000.0, &mut out_a_p, &mut sample_count);
        out_b_p.clone_from_slice(&mut out_a_p);
        //dbg!(&sample_count);
        jack::Control::Continue
    };
    let process = jack::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = client.activate_async(Notifications, process).unwrap();

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    active_client.deactivate().unwrap();
}

struct Notifications;

impl jack::NotificationHandler for Notifications {
    fn thread_init(&self, _: &jack::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: jack::ClientStatus, reason: &str) {
        println!(
            "JACK: shutdown with status {:?} because \"{}\"",
            status, reason
        );
    }

    fn freewheel(&mut self, _: &jack::Client, is_enabled: bool) {
        println!(
            "JACK: freewheel mode is {}",
            if is_enabled { "on" } else { "off" }
        );
    }

    fn sample_rate(&mut self, _: &jack::Client, srate: jack::Frames) -> jack::Control {
        println!("JACK: sample rate changed to {}", srate);
        jack::Control::Continue
    }

    fn client_registration(&mut self, _: &jack::Client, name: &str, is_reg: bool) {
        println!(
            "JACK: {} client with name \"{}\"",
            if is_reg { "registered" } else { "unregistered" },
            name
        );
    }

    fn port_registration(&mut self, _: &jack::Client, port_id: jack::PortId, is_reg: bool) {
        println!(
            "JACK: {} port with id {}",
            if is_reg { "registered" } else { "unregistered" },
            port_id
        );
    }

    fn port_rename(
        &mut self,
        _: &jack::Client,
        port_id: jack::PortId,
        old_name: &str,
        new_name: &str,
    ) -> jack::Control {
        println!(
            "JACK: port with id {} renamed from {} to {}",
            port_id, old_name, new_name
        );
        jack::Control::Continue
    }

    fn ports_connected(
        &mut self,
        _: &jack::Client,
        port_id_a: jack::PortId,
        port_id_b: jack::PortId,
        are_connected: bool,
    ) {
        println!(
            "JACK: ports with id {} and {} are {}",
            port_id_a,
            port_id_b,
            if are_connected {
                "connected"
            } else {
                "disconnected"
            }
        );
    }

    fn graph_reorder(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: graph reordered");
        jack::Control::Continue
    }

    fn xrun(&mut self, _: &jack::Client) -> jack::Control {
        println!("JACK: xrun occurred");
        jack::Control::Continue
    }
}
