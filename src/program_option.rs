use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(about = "cli application for the hello-world program", long_about = None) ]
pub struct Args {
    /// Program mode (0 - Create account, 1 - Resize account). If not set will be asked
    #[arg(short, long)]
    pub mode: Option<u8>,
    /// Seed for PDA. If not set will be asked
    #[arg(short = 's', long)]
    pub seed: Option<String>,
    /// New account size (ignored if mode = 0). If not set will be asked
    #[arg(short = 'S', long)]
    pub size: Option<u64>,
}