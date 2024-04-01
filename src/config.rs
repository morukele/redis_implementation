#[derive(Clone, Debug, Default)]
pub enum Mode {
    #[default]
    Master,
    Slave(Vec<String>),
}

#[derive(Clone, Debug, Default)]
pub struct Server {
    pub host: String,
    pub port: usize,
    pub mode: Mode,
    pub master_replid: Option<String>,
    pub master_repl_offset: usize,
}

impl Server {
    pub fn new(
        host: String,
        port: usize,
        mode: Mode,
        master_replid: Option<String>,
        master_repl_offset: usize,
    ) -> Server {
        Self {
            host,
            port,
            mode,
            master_replid,
            master_repl_offset,
        }
    }
}
