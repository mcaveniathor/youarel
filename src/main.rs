
//#![allow(unused_imports)]
//
//

#![feature(test)]
extern crate test;
extern crate anyhow;
#[macro_use] extern crate tracing;
extern crate clap_verbosity_flag as cvf;
extern crate tracing_subscriber;
use tracing_subscriber::{prelude::*, fmt::{self, time}};
extern crate clap;
use clap::Parser;
extern crate tokio;
extern crate base64;
extern crate blake3;
extern crate url;
#[macro_use] extern crate lazy_static;
extern crate sled;
extern crate tower_http;
use tower_http::services::ServeDir;
use std::sync::Arc;
extern crate directories;
extern crate rcgen;
extern crate rustls;
use anyhow::Result;
pub mod util;
use util::*;
pub mod handlers;
use handlers::*;
pub mod transport;
pub mod encoding;
use std::net::SocketAddr;
#[cfg(feature="axum")]
extern crate axum;
#[cfg(feature="axum")]
use axum::{Router, routing::{get, get_service},};
use axum_server::tls_rustls::RustlsConfig;

lazy_static! {
    pub static ref ENCODED_LEN: usize = {
        let cli = Cli::parse();
        cli.length
    };
}



#[cfg(feature = "axum")]
async fn run_axum(cli: Cli, db: Arc<sled::Db>) -> Result<()> {
    let state = AppState { db, hostname: cli.hostname.clone(), port: cli.port };
    let app = Router::new()
        .route("/", get(root).post(accept_form))
        .route("/:key", get(redirect))
        .with_state(state)
        .nest_service("/assets", get_service(ServeDir::new("./assets")));
    let addr = SocketAddr::new(cli.address, cli.port);
    #[cfg(not(feature = "tls"))]
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let (_conf, cert_chain, priv_key) = transport::configure_server_tls(cli.cert.as_ref(), cli.key.as_ref())?;
    let server_config = RustlsConfig::from_der(cert_chain.iter().map(|c| c.as_ref().to_vec()).collect(), priv_key.0).await?;
    info!("Binding to {}", addr);
    tokio::select! {
        res = {
            #[cfg(feature = "tls")] {
                axum_server::bind_rustls(addr, server_config)
                .serve(app.into_make_service())
            }
            #[cfg(not(feature = "tls"))]
            {
                axum::serve(listener, app) 
            } 
        } => {
                match res {
                    Ok(_) => debug!("Serve task completed without error."),
                    Err(e) => error!("Serve task exited with error: {:?}", e),
                }
        }
        _ = tokio::signal::ctrl_c() => {
            warn!("ctrl-c signal received, shutting down gracefully.");
        }
    }
    Ok(())
}


#[tokio::main]
async fn run(cli: Cli) -> Result<()> {
    tracing::debug!("Opening database at {}", &cli.db);
    let db = Arc::new(sled::open(&cli.db)?);
    //let (cert_chain, private_key) = transport::configure_server_cert(cli.cert.as_ref(), cli.key.as_ref())?;
    {
        #[cfg(feature = "axum")]
        run_axum(cli, db.clone())
    }.await?;
    
    let bytes_flushed = db.flush()?;
    debug!("Flushed {} bytes to disk", bytes_flushed);
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let fmt_layer = {
        let layer = fmt::layer()
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_line_number(true)
            .with_file(true)
            .with_timer(time::uptime());
        if cli.compact {
            layer.compact().boxed()
        }
        else if cli.pretty {
            layer.pretty().boxed()
        }
        else {
            layer.boxed()
        }
    }
    .with_filter(verbosity_to_level(cli.verbose.log_level().unwrap_or(cvf::Level::Info)));
    let subscriber = tracing_subscriber::registry()
        .with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber.");
    info!("Initialized logger.");
    run(cli).map_err(|e| { error!("{}",e); e})?;
    Ok(())
}

