use clap::ValueEnum;

#[derive(ValueEnum, Clone, Debug)]
// #[repr(u8)]
pub enum TransactionType {
    Create = 0,
    Resize = 1,
    Transfer = 2,
    TransferFrom = 3,
    Allocate = 4,
}
