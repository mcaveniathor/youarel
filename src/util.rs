use cvf::{Verbosity,InfoLevel};
use clap::Parser;
use tracing_subscriber::filter::LevelFilter;
use directories::ProjectDirs;
use sled::Db;
use std::{net::IpAddr, path::PathBuf, sync::Arc};
use blake3::Hasher;
use base64::Engine;
use serde::{Serialize,Deserialize};
use url::Url;


/// Hash `s` with blake3, Base64 encode (URL-safe with no padding) the hash, and truncate to ENCODED_LENGTH characters. 
pub fn encode(s: impl AsRef<str>, length: usize) -> String {
    let mut hasher = Hasher::new();
    hasher.update(s.as_ref().as_bytes());
    let hash = hasher.finalize();
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let bytes_len = 3 * (length/4); // length of the truncated length which yields a base64
    trace!("length: {}, bytes_len: {}", length, bytes_len);
    let mut out_buf = engine.encode(hash.as_bytes());
    out_buf.truncate(length);
    out_buf
}


#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub hostname: String,
    pub port: u16,
}


#[derive(Serialize,Deserialize)]
pub struct ShortenReq {
    /// The URL to be shortened
    pub long_url: Url,
    /// The desired number of characters in the shorted URL
    pub length: Option<usize>,
}


#[derive(Debug,Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    #[arg(long)]
    /// Use compact formatting for log messages.
    pub compact: bool,

    #[arg(long)]
    /// Whether log messages should be pretty printed. The --compact option will override this if
    /// set.
    pub pretty: bool,

    #[arg(short,long, default_value="::1")]
    /// The address to bind to
    pub address: IpAddr,

    #[arg(short,long, default_value="3000")]
    /// The port to bind to
    pub port: u16,

    #[arg(short, long, default_value="8")]
    /// The number of base64 characters used in shortened URLs. A smaller number increases the
    /// chances of collisions, but that small chance is probably worth it considering this is a URL
    /// shortener
    pub length: usize,

    #[arg(short, long, default_value_t = get_project_data_dir())]
    /// Path to the database root. Defaults to the appropriate data directory 
    /// according to XDG/Known Folder/Standard directories specifications based on OS
    pub db: String,
    /// TLS private key in DER format
    #[arg(short = 'k', long = "key", requires = "cert")]
    pub key: Option<PathBuf>,
    /// TLS certificate in DER format
    #[arg(short = 'c', long = "cert", requires = "key")]
    pub cert: Option<PathBuf>,
    /// Enable stateless retries
    #[arg(long = "stateless-retry")]
    pub stateless_retry: bool,

    #[arg(long, default_value = "localhost")]
    pub hostname: String,
}


pub fn verbosity_to_level(level: cvf::Level) -> LevelFilter {
    use cvf::Level::*;
    match level {
        Error => LevelFilter::ERROR,
        Warn => LevelFilter::WARN,
        Info => LevelFilter::INFO,
        Debug => LevelFilter::DEBUG,
        Trace => LevelFilter::TRACE,
    }
}

fn get_project_data_dir() -> String {
    if let Some(proj_dirs) = ProjectDirs::from("com", "mcaveniathor", "youarel") {
        proj_dirs.data_dir().display().to_string()
    }
    else {
        "db".into()
    }
}
/*
pub fn get_cert_key(cert_path: impl AsRef<PathBuf>, key_path: impl AsRef<PathBuf>) -> Result<
*/
