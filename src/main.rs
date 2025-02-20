use std::cmp::{max, min};
use std::io::{self, BufRead};
use std::sync::mpsc;
use std::thread::{self, sleep};
use std::time::Duration;

use clap::Parser;

mod args;
use crate::args::Args;

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // Use channels to achieve nonblocking reads from stdin
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut lines = io::stdin().lock().lines();
        loop {
            let line = lines.next().unwrap().unwrap();
            let ms = match line.trim().parse::<u32>() {
                Ok(ms) => ms,
                _ => 0,
            };
            tx.send(ms).unwrap();
        }
    });

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
                println!("1");
            }
            sleep(pulse);
        }
        if !remainder.is_zero() {
            if is_on {
                is_on = false;
                println!("0");
            }
            sleep(remainder);
        }

        // Consume all commands
        pulse_ms = 0;
        while let Ok(ms) = rx.try_recv() {
            // pulse must be no longer than period and not between (0, min)
            pulse_ms = match ms {
                0 => 0,
                _ => min(max(args.min_pulse_ms, ms), args.period_ms),
            };
        }
        eprintln!("pulsing for {} ms", pulse_ms);
    }
}
