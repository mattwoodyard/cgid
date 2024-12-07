use std::{net::ToSocketAddrs, os::fd::FromRawFd};

use tokio::net::TcpListener;

pub fn startup_systemd(_systemd_name: &str) -> Result<TcpListener, String> {
    match std::env::var("LISTEN_FDS") {
        Ok(fds) if fds == "1" => {
            let raw_fd = 3; // Systemd provides the socket on file descriptor 3
            unsafe {
                TcpListener::from_std(std::net::TcpListener::from_raw_fd(raw_fd))
                    .map_err(|e| format!("from_raw_fd: {:?}", e))
            }
        }
        _ => {
            eprintln!("This program must be run via systemd socket activation.");
            Err("No sockets provided".to_string())
        }
    }
}

pub async fn startup_persistent_server(listen_spec: &str) -> Result<TcpListener, String> {
    let addr = listen_spec
        .to_socket_addrs()
        .map_err(|e| format!("Error in bind addr"))?;
    for in_addr in addr {
        let listener = TcpListener::bind(in_addr)
            .await
            .map_err(|e| format!("Error in bind: {:?}", e))?;

        return Ok(listener);
    }

    Err("Failure to bind".to_string())
}
