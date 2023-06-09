use cvf::{Verbosity,InfoLevel};
use clap::Parser;
use tracing_subscriber::filter::LevelFilter;
use directories::ProjectDirs;
use sled::Db;
use std::{net::IpAddr, path::PathBuf, sync::Arc };
use blake3::Hasher;
use base64::Engine;
use serde::{Serialize,Deserialize};
use chrono::{DateTime,Utc};
use url::Url;


/// Serialize value of type `S` and insert into the database, returning the existing value if successfully
/// deserialized into a `D`
pub fn ser_insert<K, S, D>(db: Arc<Db>, key: K, v: &S) -> anyhow::Result<Option<D>>
where
    K: AsRef<[u8]>,
    S: Serialize,
    D: for <'a> Deserialize<'a>,
{
    let serialized = bincode::serialize(v)?;
    let old = db.insert(key, serialized)?;
    match old {
        Some(old_bytes) => {
            let old_decoded: D = bincode::deserialize(&old_bytes[..])?;
            Ok(Some(old_decoded))
        },
        _ => {
            Ok(None)
        }
    }
}

/// Retrieve a value from the database and attempt to deserialize it into a `D`
pub fn de_get<K, D>(db: Arc<Db>, key: K) -> anyhow::Result<Option<D>>
where
    K: AsRef<[u8]>,
    D: for <'a> Deserialize <'a>,
{
    let val = db.get(key)?;
    match val {
        Some(val_bytes) => {
            let decoded = bincode::deserialize(&val_bytes[..])?;
            Ok(Some(decoded))
        },
        _ => Ok(None)
    }
}

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
    pub default_length: usize,
}


#[derive(Serialize, Deserialize)]
pub struct ShortenEntry {
    /// Time of creation
    pub created: DateTime<Utc>,
    /// The original URL
    pub long_url: Url,
    /// Remaining number of accesses before expiration, if specified
    pub accesses: Option<usize>,
}

impl ShortenEntry {
    fn new(req: ShortenReq) -> ShortenEntry {
        ShortenEntry {
            created: Utc::now(),
            long_url: req.long_url,
            accesses: req.accesses,
        }
    }
}
impl From<ShortenReq> for ShortenEntry {
    fn from(req: ShortenReq) -> ShortenEntry {
        ShortenEntry::new(req)
    }
}


#[derive(Serialize,Deserialize)]
pub struct ShortenReq {
    /// The URL to be shortened
    pub long_url: Url,
    /// The desired number of characters in the shorted URL
    pub length: Option<usize>,
    /// Remove the shortened URL after this many accesses
    pub accesses: Option<usize>,
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

    #[arg(short,long, default_value="443")]
    /// The port to bind to
    pub port: u16,

    #[arg(short, long, default_value="8")]
    /// The number of base64 characters used in shortened URLs. A smaller number increases the
    /// chances of collisions 
    pub length: usize,

    #[arg(short, long, default_value_t = get_project_data_dir())]
    /// Path to the database root. Defaults to the appropriate data directory 
    /// according to XDG/Known Folder/Standard directories specifications based on OS
    pub db: String,
    /// Path to file containing TLS private key in DER format
    #[arg(short = 'k', long = "key", requires = "cert")]
    pub key: Option<PathBuf>,
    /// Path to file containing TLS certificate in DER format
    #[arg(short = 'c', long = "cert", requires = "key")]
    pub cert: Option<PathBuf>,
    /*
    /// Enable stateless retries
    #[arg(long = "stateless-retry")]
    pub stateless_retry: bool,
    */

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
