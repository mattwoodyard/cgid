use cgid::dispatcher::*;
use cgid::request_parser::*;
use cgid::types::*;
use async_std;

use std::env::args;
use tide::prelude::*;
use tide::Request;

use std::sync::Arc;

#[async_std::main]
async fn main() -> tide::Result<()> {

  let mut _arg = args();
  _arg.next();
  let arg = _arg.next().unwrap();
  let config = Arc::new(Config {
    auth_script: None,
    script_root: arg
  });

  let mut app = tide::new();

  let c2 = config.clone();
  app.at("/:script").post(move |r| dispatcher(c2.clone(), r));
  app.at("/:script").get(move |r| dispatcher(config.clone(), r));
  app.listen("127.0.0.1:8081").await?;
  Ok(())
}
