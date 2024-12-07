use std::path::PathBuf;

use futures_util::{Stream, TryFutureExt, TryStreamExt};
use http_body_util::BodyExt;
use hyper::{
    body::{Body, Bytes, Frame, Incoming},
    Request, Response,
};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command},
    sync::mpsc::Sender,
    task::JoinHandle,
};
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Debug)]
pub enum ProcessError {
    InvalidPath(std::io::Error),
    RunPathNotInRoot,
    RunPathNotExists,
    ExecutionError(std::io::Error),
    BuildResponse(hyper::http::Error),
}

pub struct UpstreamProcessBuilder {
    root: PathBuf,
    _enable_params: bool,
}
impl UpstreamProcessBuilder {
    pub fn new(rootpath: &str) -> Result<UpstreamProcessBuilder, ProcessError> {
        let path = PathBuf::from(rootpath);
        if !path.exists() {
            log::error!("root path: {:?} doesn't exist", path);
            Err(ProcessError::InvalidPath(std::io::Error::last_os_error()))
        } else {
            Ok(UpstreamProcessBuilder {
                root: path,
                _enable_params: false,
            })
        }
    }

    pub fn build<B>(&self, req: &Request<B>) -> Result<UpstreamProcess, ProcessError> {
        let uri_path = req.uri().path().trim_start_matches('/');
        let fpath = self.root.join(uri_path);
        let fpath = fpath.canonicalize().map_err(ProcessError::InvalidPath)?;

        if !fpath.ancestors().any(|f| f == self.root) {
            log::error!("path: {:?}, not in root path: {:?}", fpath, self.root);
            Err(ProcessError::RunPathNotInRoot)
        } else if !fpath.exists() {
            log::error!("path: {:?}, doesn't exist", fpath);
            Err(ProcessError::RunPathNotExists)
        } else {
            Ok(UpstreamProcess {
                exec_path: fpath,
                exec_args: vec![],
            })
        }
    }
}

pub struct UpstreamProcess {
    pub exec_path: PathBuf,
    pub exec_args: Vec<String>,
}

impl UpstreamProcess {
    pub async fn run(
        self,
        req: Request<Incoming>,
        joiner: Sender<JoinHandle<()>>,
    ) -> Result<Response<Box<impl Body<Data = Bytes, Error = std::io::Error>>>, ProcessError> {
        log::info!("launch: {:?} : {:?}", self.exec_path, self.exec_args);
        let child = Command::new(self.exec_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();
        match child {
            Ok(mut child_process) => {
                let stdin = child_process.stdin.take();
                let stdout = child_process.stdout.take();

                // TODO(matt) - log stderr
                //let stderr = child_process.stderr.take();

                tokio::spawn(launch_stdin(stdin, req));
                let resp_body = stream_stdout(stdout)?;

                let chld_wait = tokio::spawn(notify_child_end(child_process));
                if let Err(e) = joiner.send(chld_wait).await {
                    log::error!("Error sending join handle {:?}", e);
                }

                Response::builder()
                    .status(200)
                    .body(Box::new(resp_body))
                    .map_err(ProcessError::BuildResponse)
            }
            Err(e) => Err(ProcessError::ExecutionError(e)),
        }
    }
}

async fn notify_child_end(mut child_process: Child) {
    let proc_result = child_process.wait();
    let _ = proc_result
        .map_ok(|ex| {
            log::info!("exit {:?}", ex);
        })
        .map_err(|er| {
            log::error!("exit {:?}", er);
        })
        .await;
}

async fn launch_stdin(process: Option<ChildStdin>, mut req: Request<Incoming>) {
    if let Some(mut stdin) = process {
        let rbody = req.body_mut();
        loop {
            match rbody.frame().await {
                Some(Ok(frame)) => match frame.data_ref() {
                    Some(data) => {
                        let _ = stdin.write_all(data).await;
                    }
                    None => todo!(),
                },

                Some(Err(_)) => todo!(),
                None => {
                    break;
                }
            }
        }
    }
}

pub fn stream_stdout(
    process: Option<ChildStdout>,
) -> Result<
    http_body_util::StreamBody<impl Stream<Item = Result<Frame<Bytes>, std::io::Error>>>,
    ProcessError,
> {
    if let Some(stdout) = process {
        let rdr = BufReader::new(stdout);
        let my_stream_of_bytes = FramedRead::new(rdr, BytesCodec::new());
        let my_stream_of_bytes = my_stream_of_bytes.map_ok(|b| Frame::data(b.freeze()));
        Ok(http_body_util::StreamBody::new(my_stream_of_bytes))
    } else {
        todo!()
    }
}

#[test]
fn test_path_magic() {}
