use anyhow::{anyhow,bail,Context};
use rustls::{PrivateKey,Certificate};
use directories::ProjectDirs;
use std::{io::Write, path::Path, fs::{self, File}};

pub fn configure_server_tls(cert_path: Option<impl AsRef<Path>>, key_path: Option<impl AsRef<Path>>) -> anyhow::Result<(rustls::ServerConfig, Vec<Certificate>, PrivateKey)> {
    let (cert_chain, priv_key) = {
        if let (Some(cp), Some(kp)) = (cert_path, key_path) {
            let cert = fs::read(cp.as_ref())
                .context(format!("Failed to read certificate chain from file {}",cp.as_ref().display()))?;
            let key = fs::read(kp.as_ref())
                .context(format!("Failed to read private key from file {}",kp.as_ref().display()))?;
            info!("Successfully read certificate and key from {} and {}", cp.as_ref().display(), kp.as_ref().display());
            let cert_chain = vec![Certificate(cert)];
            let priv_key = PrivateKey(key);
            (cert_chain, priv_key)
        }
        else {
            let pd = ProjectDirs::from("com", "mcaveniathor", "youarel").unwrap();
            let pd = pd.data_local_dir();
            let (cp, kp) = (pd.join("cert.der"), pd.join("key.der"));
                match fs::read(&cp).map_err(|e| anyhow!(e)).and_then(|x| Ok((x, fs::read(&kp).context("Failed to read private key")?)))  {
                Ok((c, k)) => {
                    info!("Read certificate and private key from {} and {}",cp.display(), kp.display());
                    (vec![Certificate(c)], PrivateKey(k))
                },
                Err(e) if e.downcast_ref::<std::io::Error>().unwrap().kind() == std::io::ErrorKind::NotFound => {
                    info!("No path to certificate chain/private key given and none found at {}, {}. Generating self-signed certificate.", &cp.display(), &kp.display());
                    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).context("Failed to generate self-signed certificate chain.")?;
                    let cert_der = cert.serialize_der().context("Failed to serialize self-signed certificate as DER")?;
                    let priv_key = cert.serialize_private_key_der();
                    let (mut cert_file, mut key_file) = (File::create_new(&cp)?, File::create_new(&kp)?);
                    cert_file.write_all(&cert_der)?;
                    key_file.write_all(&priv_key)?;
                    info!("Wrote self-signed certificate to {} and private key to {}", cp.display(), kp.display());
                    let priv_key = rustls::PrivateKey(priv_key);
                    let cert_chain = vec![rustls::Certificate(cert_der)];
                    (cert_chain, priv_key)
                },
                Err(e) => { bail!("Failed to read certificate: {}", e); }
            }
        }
    };
    let server_config = rustls::ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain.clone(), priv_key.clone()).context("Failed to create server TLS configuration")?;
    Ok((server_config, cert_chain, priv_key))
}
