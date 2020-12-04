use clap::Clap;
use serde::{Deserialize, Serialize};
use tide::http::headers::{HeaderName, HeaderValue, HeaderValues};
use tide::{Body, Request, Response, StatusCode};

#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct AcmeConfig {}

#[clap(version = "1.0", author = "Matt Woodyard <matt@mattwoodyard.com>")]
#[derive(Debug, Serialize, Deserialize, Clap)]
pub struct Config {
    pub script_root: String,
    #[clap(long)]
    pub auth_script: Option<String>,
    #[clap(long)]
    pub cert: Option<String>,
    #[clap(long)]
    pub priv_key: Option<String>,
    // pub acme_config: Option<AcmeConfig>,
    #[clap(long)]
    pub bind_address: Option<String>,
    #[clap(long)]
    pub bind_port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EncType {
    Raw,
    Base64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessRequest {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    encoding: EncType,
    body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResponse {
    code: u16,
    headers: Vec<(String, String)>,
    encoding: EncType,
    body: Option<String>,
}

pub fn to_tide_response(p: ProcessResponse) -> Result<Response, CgidError> {
    let builder = Response::builder(p.code);
    let h = p
        .headers
        .iter()
        .map(|(k, v)| {
            let hn = HeaderName::from(k.as_str());
            let hv = HeaderValue::from_bytes(Vec::from(v.as_bytes()));
            hv.map(|h| (hn, h)).map_err(From::from)
        })
        .collect::<Result<Vec<(HeaderName, HeaderValue)>, CgidError>>()?;

    let builder = h.into_iter().fold(builder, |acc, (k, v)| acc.header(k, v));

    let builder = if let Some(body) = p.body {
        match p.encoding {
            EncType::Base64 => base64::decode(&body).map(|b| builder.body(b.as_slice()))?,
            EncType::Raw => builder.body(body),
        }
    } else {
        builder
    };

    Ok(builder.build())
}

impl ProcessRequest {
    pub async fn from<A>(mut r: Request<A>) -> Result<ProcessRequest, CgidError> {
        let headers: Vec<(String, String)> = r
            .header_names()
            .map(|n| (n.to_string(), r.header(n).unwrap().to_string()))
            .collect();

        let etype = match r
            .content_type()
            .map(|c| c.essence() == "application/json")
            .unwrap_or(false)
        {
            true => EncType::Raw,
            _ => EncType::Base64,
        };

        let body = if r.is_empty().unwrap_or(true) {
            None
        } else {
            Some(match etype {
                EncType::Base64 => {
                    let b = r.body_bytes().await?;
                    base64::encode(&b).to_string()
                }
                EncType::Raw => {
                    let b = r.body_json::<serde_json::Value>().await?;
                    let o = serde_json::to_string(&b)?;
                    o
                }
            })
        };

        Ok(ProcessRequest {
            method: r.method().to_string(),
            url: r.url().to_string(),
            headers,
            encoding: etype,
            body,
        })
    }
}

#[derive(Debug)]
pub enum CgidError {
    NonExistantScript,
    InvalidPath,
    ScriptNoExec,
    IoError(std::io::Error),
    JsonError(serde_json::error::Error),
    NoChildStdout,
    NoChildStdin,
    ExecError(u8),
    TideError(tide::Error),
    Base64(base64::DecodeError),
    Spawn(std::io::Error)
}

impl From<std::io::Error> for CgidError {
    fn from(e: std::io::Error) -> CgidError {
        CgidError::IoError(e)
    }
}

impl From<base64::DecodeError> for CgidError {
    fn from(e: base64::DecodeError) -> CgidError {
        CgidError::Base64(e)
    }
}
impl From<tide::Error> for CgidError {
    fn from(e: tide::Error) -> CgidError {
        CgidError::TideError(e)
    }
}
impl From<serde_json::error::Error> for CgidError {
    fn from(e: serde_json::error::Error) -> CgidError {
        CgidError::JsonError(e)
    }
}
