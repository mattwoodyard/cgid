use async_std;
use cgid::dispatcher::*;
use cgid::types::*;

use tide::prelude::*;
use tide_rustls::TlsListener;

use clap::Clap;
use std::sync::Arc;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();
    tide::log::start();

    let config: Arc<Config> = Arc::new(Config::parse());

    let c1 = config.clone();
    let c2 = config.clone();
    app.at("/:script").post(move |r| dispatcher(c2.clone(), r));
    app.at("/:script").get(move |r| dispatcher(c1.clone(), r));

    let enable_tls = config.cert.is_some() && config.priv_key.is_some();
    let bind_addr = format!(
        "{}:{}",
        config
            .bind_address
            .as_ref()
            .map(|e| e.as_str())
            .unwrap_or("127.0.0.1"),
        config.bind_port.unwrap_or(8080)
    );

    tide::log::info!("listening on: {}, tls enabled: {}", bind_addr, enable_tls);
    tide::log::info!("serving: {}", config.script_root);
    tide::log::info!("request based auth: {:?}", config.auth_script);

    if enable_tls {
        let listener = TlsListener::build()
            .addrs(bind_addr)
            .cert(config.cert.clone().unwrap())
            .key(config.priv_key.clone().unwrap());
        app.listen(listener).await?
    } else {
        app.listen(bind_addr).await?;
    }

    Ok(())
}
