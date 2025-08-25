mod transaction_type;

pub(crate) use {clap::Parser, transaction_type::TransactionType};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(about = "cli application for the hello-world program", long_about = None) ]
pub struct Args {
    /// Path to signer keypair file
    #[arg(long)]
    pub keypair_path: String, //pub mode: u8,
    /// Program mode (Create - Create account, Resize - Resize account, Send - Send lamports).
    #[arg(long)]
    pub mode: TransactionType, //pub mode: u8,
    /// Seed for PDA. If not set will be asked
    #[arg(long)]
    #[arg(long, required_if_eq("mode", "create"))]
    #[arg(long, required_if_eq("mode", "resize"))]
    #[arg(long, required_if_eq("mode", "transferfrom"))]
    pub seed: Option<String>,
    /// New account size (ignored if mode = 0).
    #[arg(long, required_if_eq("mode", "resize"))]
    pub size: Option<u64>,


    // /// Source account Id (From which transfer will be done)
    // #[arg(long, required_if_eq("mode", "send"))]
    // pub source: Option<String>,
    /// Destination account Id (To which transfer will be done)
    #[arg(long, required_if_eq("mode", "transfer"))]
    #[arg(long, required_if_eq("mode", "transferfrom"))]
    pub to: Option<String>,
    /// Lamports to send.
    #[arg(long, required_if_eq("mode", "transfer"))]
    #[arg(long, required_if_eq("mode", "transferfrom"))]
    pub amount: Option<u64>,
}
