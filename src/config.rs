#[derive(Clone, Debug, Default)]
pub enum Mode {
    #[default]
    Master,
    Slave(Vec<String>),
}
