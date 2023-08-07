use rustls_pemfile::{Item, read_all};
use std::sync::Arc;
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{Read, BufReader};

use std::env::current_dir;

use crate::route_table::RouteTable;
use crate::request_state;
use crate::body::Body;
use hyper::Request;


pub type Acceptor = tokio_rustls::TlsAcceptor;

#[derive(Clone, Debug)]
pub struct Config {
    pub root: PathBuf,
    pub port: u16,
    pub host: String,

    pub keypair: Option<KeyReader>,
}

impl Config {

    pub fn from_routes<R: AsRef<RouteTable>>(mut self, table: &R) -> Self {
        todo!();
        // let route_table = table.as_ref();

        // if self.keypair == None {
        //     if let Some(_) = route_table.get("ssl") {
        //         let route = route_table.get("ssl").unwrap();
        //         let dummy_request = Request::new(hyper::Body::empty());
        //         let mut state = request_state::RequestState::new(&route, dummy_request, &route_table);
        //         for command in &route.commands {
        //             command.run(&mut state);
        //         }

        //         println!("{:?}", state.body);

        //         match state.body {
        //             Body::Txt(txt) => {
        //                 let keyreader = KeyReader::new().read_pem(&mut txt.as_bytes());
        //                 self.keypair = Some(keyreader);
        //             },
        //             Body::Raw(bytes) => {
        //                 let keyreader = KeyReader::new().read_pem(&mut bytes.as_slice());
        //                 self.keypair = Some(keyreader);
        //             },
        //             _ => { println!("could not load key file") },
        //         }
        //     }
        // }

        // self
    }

    pub fn from_args(matches: &clap::ArgMatches) -> Self {
        todo!();
        // let port = matches.value_of("port").unwrap_or("4000");
        // let host = matches.value_of("host").unwrap_or("0.0.0.0");

        // let keypair = match (matches.value_of("cert"), matches.value_of("key")) {
        //     (Some(cert), Some(key)) => {
        //         let keyreader = KeyReader::new()
        //             .read_pem(&mut File::open(&cert).unwrap())
        //             .read_pem(&mut File::open(&key).unwrap());
        //         Some(keyreader)

        //     },
        //     _ => None,
        // };

        // // Determine the local path to serve files out of 
        // let path = Path::new(matches.value_of("PATH").unwrap_or("."));

        // // if the path given has a root, ie. /home/www/public, use it as is,
        // // if not, ie. server/public join it to the end of the current directory
        // let path_abs = match path.has_root() {
        //     true => path.to_path_buf(),
        //     false => current_dir().unwrap().join(path),
        // }.canonicalize().unwrap();

        // Self {
        //     root: path_abs,
        //     port: port.parse::<u16>().unwrap(),
        //     host: host.to_string(),
        //     keypair
        // }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyReader {
    key: Option<Vec<u8>>,
    certs: Vec<Vec<u8>>,
}

impl KeyReader {
    pub fn new() -> Self { 
        Self {key: None, certs: Vec::new() }
    }

    pub fn read_pem<R: Read>(mut self, input: &mut R) -> Self {
        let mut buf_read = BufReader::new(input);
        let items = read_all(&mut buf_read).unwrap();
        for item in items {
            match item {
                Item::X509Certificate(cert) => self.certs.push(cert),
                Item::RSAKey(key) => self.key = Some(key),
                Item::PKCS8Key(key) => self.key = Some(key),
                Item::ECKey(key) => self.key = Some(key),
                _ => todo!(),
            }
        }

        self
    }

    pub fn into_tls_acceptor(self) -> Result<Acceptor, ()> {

        if self.certs.len() == 0 { return Err(()) }
        let key_bytes = self.key.ok_or(())?;
        let cert_bytes = self.certs.into_iter().next().ok_or(())?;

        let key = PrivateKey(key_bytes.into());
        let cert = Certificate(cert_bytes.into());

        let server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .unwrap();

        Ok(Arc::new(server_config).into())
    }
}
