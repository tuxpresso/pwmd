use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
pub struct Args {
    /// Period in millis
    #[arg(short)]
    pub period_ms: u32,

    /// Minimum pulse width in millis
    #[arg(short)]
    pub min_pulse_ms: u32,

    /// Path to gpio sysfs
    #[arg(short)]
    pub gpio_path: String,

    /// Address to bind socket to
    #[arg(short)]
    pub bind_address: String,
}
