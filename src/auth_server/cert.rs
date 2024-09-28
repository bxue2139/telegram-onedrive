/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use axum_server::tls_rustls::RustlsConfig;
use proc_macros::{add_context, add_trace};
use rcgen::{generate_simple_self_signed, CertifiedKey};
use std::path::Path;

use crate::error::{Error, Result};

#[add_context]
#[add_trace]
pub async fn get_rustls_config() -> Result<RustlsConfig> {
    let cert_path = Path::new("ssl/server.crt");
    let key_path = Path::new("ssl/server.key");

    let config = if cert_path.exists() && key_path.exists() {
        tracing::info!("auth server uses cert from file");

        RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .map_err(|e| Error::new("failed to create rustls config from pem file").raw(e))?
    } else {
        tracing::info!("auth server uses self signed cert");

        let (cert, key) = gen_self_signed_cert()?;

        RustlsConfig::from_pem(cert, key)
            .await
            .map_err(|e| Error::new("failed to create self signed rustls config").raw(e))?
    };

    Ok(config)
}

#[add_context]
#[add_trace]
fn gen_self_signed_cert() -> Result<(Vec<u8>, Vec<u8>)> {
    let subject_alt_names = vec!["127.0.0.1".to_string(), "localhost".to_string()];

    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names)
        .map_err(|e| Error::new("failed to generate self signed cert").raw(e))?;

    Ok((
        cert.pem().into_bytes(),
        key_pair.serialize_pem().into_bytes(),
    ))
}
