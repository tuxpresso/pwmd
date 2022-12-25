use std::cmp::{max, min};
use std::fs::File;
use std::io::Write;
use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;

use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// Period in millis
    #[arg(short)]
    period_ms: u32,

    /// Minimum pulse width in millis
    #[arg(short)]
    min_pulse_ms: u32,

    /// Path to gpio sysfs
    #[arg(short)]
    gpio_path: String,

    /// Address to bind socket to
    #[arg(short)]
    bind_address: String,
}

fn read_u32(sock: &UdpSocket) -> Option<u32> {
    let mut buf = [0; 5];
    match sock.recv_from(&mut buf) {
        Ok((4, _)) => Some(
            (buf[0] as u32)
                | (buf[1] as u32) << 8
                | (buf[2] as u32) << 16
                | (buf[3] as u32) << 24,
        ),
        Ok((_, _)) => read_u32(sock), // consume invalid data
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
                println!("on");
            }
            sleep(pulse);
        }
        if !remainder.is_zero() {
            if is_on {
                is_on = false;
                gpio.write_all("0".as_bytes())
                    .expect("failed to write to gpio");
                println!("off");
            }
            sleep(remainder);
        }

        // Consume all commands
        pulse_ms = 0;
        while let Some(ms) = read_u32(&sock) {
            // pulse must be no longer than period and not between (0, min)
            pulse_ms = match ms {
                0 => 0,
                _ => min(max(args.min_pulse_ms, ms), args.period_ms),
            };
            println!("{}", pulse_ms);
        }
    }
}
