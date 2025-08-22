use clap::ValueEnum;

#[derive(ValueEnum, Clone, Debug)]
// #[repr(u8)]
pub enum ModeType {
    Create = 0,
    Resize = 1,
    Send = 2,
}
