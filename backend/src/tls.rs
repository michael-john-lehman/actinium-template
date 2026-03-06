
pub fn config(cert: &str, key: &str) -> Result<rustls::ServerConfig, Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::BufReader;
    rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();
    let mut certs_file = BufReader::new(File::open(cert)?);
    let mut key_file = BufReader::new(File::open(key)?);
    let tls_certs = rustls_pemfile::certs(&mut certs_file)
        .collect::<Result<Vec<_>, _>>()?;
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut key_file)
        .next()
        .unwrap()
        .unwrap();
    Ok(rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certs, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))?)
}
