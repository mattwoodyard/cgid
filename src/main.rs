use cgid::args::CgiDArgs;
use cgid::process::UpstreamProcessBuilder;
use cgid::startup;
use clap::Parser;
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::body::{Bytes, Incoming};

use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::{self},
};

use hyper::{service::service_fn, Request, Response};

use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let args = CgiDArgs::parse();
    let (join_send, mut join_recv) = mpsc::channel(1024);

    if args.socket_name.is_none() && args.listen_addr.is_none() {}

    let (listener, count) = if args.socket_name.is_some() {
        (
            startup::startup_systemd(&args.socket_name.unwrap())
                .expect("initialization from socket failed"),
            1,
        )
    } else if args.listen_addr.is_some() {
        (
            startup::startup_persistent_server(&args.listen_addr.unwrap())
                .await
                .expect("bind failed"),
            -1,
        )
    } else {
        eprintln!("Require one of listen_addr or socket_name");
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Wat")
            .try_into()
            .unwrap());
    };

    let process_handler = Arc::new(
        UpstreamProcessBuilder::new(&args.root_path).expect("Error initializing proces builder"),
    );

    // Wait for all the child processes to complete
    tokio::spawn(async move {
        loop {
            match join_recv.recv().await {
                Some(_o) => {
                    log::info!("Process Exits");
                }
                None => {
                    break;
                }
            }
        }
    });

    loop {
        let process_handler = process_handler.clone();
        let join_send = join_send.clone();
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        let joinable = tokio::task::spawn(async move {
            let service = service_fn(move |req| {
                handle_request(process_handler.clone(), join_send.clone(), req)
            });
            let server = auto::Builder::new(TokioExecutor::new());
            let server = server.serve_connection(io, service);
            if let Err(e) = server.await {
                eprintln!("Failed to serve connection: {:?}", e);
            }
        });

        // Change this to me a count up, with a max requests live
        if count == 1 {
            let _ = joinable.await;
            break;
        }
    }

    Ok(())
}

async fn handle_request(
    process_handler: Arc<UpstreamProcessBuilder>,
    sender: Sender<JoinHandle<()>>,
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, std::io::Error>>, hyper::http::Error> {
    match process_handler.build(&req) {
        Ok(torun) => match torun.run(req, sender).await {
            Ok(resp) => Response::builder()
                .status(resp.status())
                .body(resp.into_body().boxed()),
            Err(err) => {
                eprintln!("{:?}", err);
                error_response(500, &format!("to run fail: {:?}", err))
            }
        },
        Err(err) => {
            eprintln!("{:?}", err);
            error_response(500, &format!("build fail: {:?}", err))
        }
    }
}

pub fn error_response(
    code: u16,
    emsg: &str,
) -> Result<Response<BoxBody<Bytes, std::io::Error>>, hyper::http::Error> {
    let error = Bytes::copy_from_slice(emsg.as_bytes());
    let body = Full::new(error);
    let body = body.map_err(|_e| std::io::Error::last_os_error());
    Response::builder().status(code).body(body.boxed())
}
