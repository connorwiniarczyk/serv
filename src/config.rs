use rustls_pemfile::{Item, read_all, read_one};
use tls_listener::TlsListener;
use std::sync::Arc;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;


pub type Acceptor = tokio_rustls::TlsAcceptor;

#[derive(Clone, Debug)]
pub struct KeyPair {
    pub key: PathBuf,
    pub cert: PathBuf,
}

impl KeyPair {
    pub fn into_tls_acceptor(&self) -> Acceptor {
        let mut key_file = BufReader::new(File::open(&self.key).unwrap());
        let key_der = match read_one(&mut key_file).unwrap().unwrap() {
            Item::X509Certificate(_) => panic!("not a valid key"),
            Item::RSAKey(key) => key,
            Item::PKCS8Key(key) => key,
            Item::ECKey(key) => key,
            _ => panic!("item not covered"),
        };

        let mut cert_file = BufReader::new(File::open(&self.cert).unwrap());
        let cert_der = match read_one(&mut cert_file).unwrap().unwrap() {
            Item::X509Certificate(cert) => cert,
            _ => panic!("not a valid certificate"),
        };

        let key = PrivateKey(key_der.into());
        let cert = Certificate(cert_der.into());

        let server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap();
         
        Arc::new(server_config).into()
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    // route_table: RouteTable,
    pub root: PathBuf,
    pub port: u16,
    pub host: String,

    pub keypair: Option<KeyPair>,
}
