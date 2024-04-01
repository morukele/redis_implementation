use rand::{distributions::Alphanumeric, Rng};
use std::fmt;

#[derive(Clone, Debug, Default)]
pub enum Mode {
    #[default]
    Master,
    Slave,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Master => write!(f, "master"),
            Mode::Slave => write!(f, "slave"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Server {
    pub host: String,
    pub port: String,
    pub mode: Mode,
    pub master_replid: Option<String>,
    pub master_repl_offset: usize,
    pub replicaof: Option<Vec<String>>,
}

impl Server {
    pub fn new(
        host: String,
        port: String,
        mode: Mode,
        master_repl_offset: usize,
        replicaof: Option<Vec<String>>,
    ) -> Server {
        // Generate random master replication Id
        let master_replid: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(40)
            .map(char::from)
            .collect();

        Self {
            host,
            port,
            mode,
            master_repl_offset,
            replicaof,
            master_replid: Some(master_replid),
        }
    }
}
