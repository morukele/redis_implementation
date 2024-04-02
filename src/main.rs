use clap::Parser;
use redis_starter_rust::{run_master, run_replica, Mode, Server};

#[derive(Parser, Debug, Clone)]
struct Config {
    #[arg(short, long, default_value_t = 6379)]
    port: usize,
    #[arg(short, long, number_of_values = 2, require_equals = false)]
    replicaof: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    // Getting server config
    let config = Config::parse();

    // Determining and set server mode
    let host = "127.0.0.1".to_string();
    let server_info = match config.replicaof {
        Some(replicaof) => Server::new(
            host,
            config.port.to_string(),
            Mode::Slave,
            0,
            Some(replicaof),
        ),
        None => Server::new(host, config.port.to_string(), Mode::Master, 0, None),
    };

    match server_info.mode {
        Mode::Master => {
            // Setting up master server
            run_master(server_info.clone()).await
        }
        Mode::Slave => match server_info.clone().replicaof {
            Some(info) => {
                // Setting up replica server and master server
                run_replica(&info[0], &info[1]).await;
                run_master(server_info.clone()).await;
            }
            None => {
                // Mode is slave but no replication info is provided
                // So it will run as if it is master
                // A better implementation will be to raise an error
                run_master(server_info.clone()).await
            }
        },
    }
}
