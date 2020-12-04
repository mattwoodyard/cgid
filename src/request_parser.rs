use async_process::Command;
use std::fs::{self, DirEntry};
use std::io;
use std::path::{Path, PathBuf};

use crate::types::*;
use tide::prelude::*;
use tide::{Body, Request, Response, StatusCode};

pub enum ParseError {
    InvalidPath,
    NonExistingScript,
    NonExecutable,
    IoError(std::io::Error),
}

pub fn parse<A>(c: &Config, req: &Request<A>) -> Result<String, ParseError> {
    let rp = req.url().path();
    println!("{:?}", rp);

    fs::read_dir(c.script_root.as_str())
        .map_err(ParseError::IoError)?
        .map(|res| res.map(|e| e.file_name()).map_err(ParseError::IoError))
        .collect::<Result<Vec<_>, ParseError>>()?
        .iter()
        .filter_map(|o| o.to_str())
        //TODO(matt) - clean this up
        .filter(|p| (String::from("/") + *p) == rp)
        .next()
        .and_then(|sf| {
            let ts = PathBuf::from(c.script_root.clone()).join(sf);
            ts.to_str().map(String::from)
        })
        .ok_or(ParseError::InvalidPath)
}
