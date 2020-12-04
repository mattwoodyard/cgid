use crate::request_parser::*;
use crate::types::*;
use async_process::{ChildStdin, ChildStdout, Command, Stdio};
// use async_std::prelude::*;
use async_std::io::prelude::ReadExt;
use futures_lite::io::AsyncWriteExt;

// use tide::prelude::*;
use tide::Request;

use std::sync::Arc;

pub async fn dispatcher<A>(c: Arc<Config>, r: Request<A>) -> tide::Result {
    println!("{:?}", c);
    match parse(&c, &r) {
        Ok(script_name) => {
            format!("target is {}", script_name);
            tide::log::info!("target is {}", script_name);
            let request = ProcessRequest::from(r).await;
            match request {
                Ok(request) => launch(&script_name, request)
                    .await
                    .and_then(to_tide_response)
                    .or_else(|e| {
                        println!("{:?}", e);
                        Ok(tide::Response::builder(500).build())
                    }),
                Err(e) => Ok(tide::Response::builder(400).build()),
            }
        }
        Err(e) => {
            match e {
                // TODO(matt) - logging
                ParseError::NonExecutable => Ok(tide::Response::builder(405).build()),
                _ => Ok(tide::Response::builder(404).build()),
            }
        }
    }
}

//TODO(matt) - pass in config stderr to log/scriptname.log

pub async fn launch(
    script_name: &str,
    request: ProcessRequest,
) -> Result<ProcessResponse, CgidError> {
    println!("launch {:?}", script_name);

    let mut child = Command::new(script_name)
        .arg(".")
        .stdout(Stdio::piped())
        // TODO(matt) - stderr to a log file
        // .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .map_err(CgidError::Spawn)?;

    let mut cstdout: ChildStdout = child.stdout.take().ok_or(CgidError::NoChildStdout)?;

    let mut cstdin: ChildStdin = child.stdin.take().ok_or(CgidError::NoChildStdin)?;

    {
        let to_child = serde_json::to_vec(&request)?;

        unsafe {
            println!("{}", String::from_utf8_unchecked(to_child.clone()));
        }

        let _sz = cstdin.write(&to_child).await?;
        let _e = cstdin.flush().await?;
        let _c = cstdin.close().await?;
        drop(cstdin);
    }

    let ec = child.status().await?;
    let mut buf: Vec<u8> = vec![];
    let _sz = cstdout.read_to_end(&mut buf).await?;
    unsafe {
        println!("{:?}", String::from_utf8_unchecked(buf.clone()));
    }

    serde_json::from_slice(buf.as_slice()).map_err(From::from)
}
