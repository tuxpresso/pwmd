use std::cmp::{max, min};
use std::fs::File;
use std::io::Write;
use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;

use ciborium::de::from_reader;
use clap::Parser;
use serde::Deserialize;

mod args;
use crate::args::Args;

#[derive(Deserialize)]
struct PwmMessage {
    pulse_ms: u32,
}

fn read_message(sock: &UdpSocket) -> Option<PwmMessage> {
    let mut buf = [0; 1024];
    match sock.recv_from(&mut buf) {
        Ok((n, _)) => match from_reader(&buf[0..n]) {
            Ok(msg) => Some(msg),
            _ => None,
        },
        _ => None,
    }
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut gpio = File::options()
        .append(true)
        .open(args.gpio_path)
        .expect("failed to open gpio");
    let sock =
        UdpSocket::bind(args.bind_address).expect("failed to bind socket");
    sock.set_nonblocking(true)
        .expect("failed to set socket to nonblocking");

    let mut is_on = false;
    let mut pulse_ms = 0;
    loop {
        // Do pwm
        let pulse = Duration::from_millis(pulse_ms.into());
        let remainder =
            Duration::from_millis((args.period_ms - pulse_ms).into());
        if !pulse.is_zero() {
            if !is_on {
                is_on = true;
                gpio.write_all("1".as_bytes())
                    .expect("failed to write to gpio");
            }
            sleep(pulse);
        }
        if !remainder.is_zero() {
            if is_on {
                is_on = false;
                gpio.write_all("0".as_bytes())
                    .expect("failed to write to gpio");
            }
            sleep(remainder);
        }

        // Consume all commands
        pulse_ms = 0;
        while let Some(msg) = read_message(&sock) {
            // pulse must be no longer than period and not between (0, min)
            pulse_ms = match msg.pulse_ms {
                0 => 0,
                ms => min(max(args.min_pulse_ms, ms), args.period_ms),
            };
            println!("{}", pulse_ms);
        }
    }
}
