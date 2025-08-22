mod mode_type;

pub(crate) use {clap::Parser, mode_type::ModeType};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(about = "cli application for the hello-world program", long_about = None) ]
pub struct Args {
    /// Program mode (Create - Create account, Resize - Resize account, Send - Send lamports). If not set will be asked
    #[arg(long)]
    pub mode: ModeType, //pub mode: u8,
    /// Seed for PDA. If not set will be asked
    #[arg(long)]
    pub seed: String,
    /// New account size (ignored if mode = 0).
    #[arg(long)]
    pub size: Option<u64>,
    /// Lamports to send.
    #[arg(long)]
    pub amount: Option<u64>,
}
