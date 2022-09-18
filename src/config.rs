use rustls_pemfile::{Item, read_one};
use std::sync::Arc;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::BufReader;

use std::env::current_dir;


pub type Acceptor = tokio_rustls::TlsAcceptor;

#[derive(Clone, Debug)]
pub struct Config {
    pub root: PathBuf,
    pub port: u16,
    pub host: String,

    pub keypair: Option<KeyPair>,
}

impl Config {
    pub fn from_args(matches: &clap::ArgMatches) -> Self {
        let port = matches.value_of("port").unwrap_or("4000");
        let host = matches.value_of("host").unwrap_or("0.0.0.0");

        let keypair = match (matches.value_of("cert"), matches.value_of("key")) {
            (Some(cert), Some(key)) => Some(KeyPair {
                cert: Path::new(&cert).to_path_buf(),
                key: Path::new(&key).to_path_buf(),
            }),
            (None, None) => None,

            // if one is set but not the other
            _ => panic!("please specify both a key and a cert or neither"),
        };

        // Determine the local path to serve files out of 
        let path = Path::new(matches.value_of("PATH").unwrap_or("."));

        // if the path given has a root, ie. /home/www/public, use it as is,
        // if not, ie. server/public join it to the end of the current directory
        let path_abs = match path.has_root() {
            true => path.to_path_buf(),
            false => current_dir().unwrap().join(path),
        }.canonicalize().unwrap();

        Self {
            root: path_abs,
            port: port.parse::<u16>().unwrap(),
            host: host.to_string(),
            keypair
        }
    }
}

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

