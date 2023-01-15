use std::collections::HashMap;
use url::Url;

use hyper;
use hyper::{Request, Response};
use crate::route_table::{Route, RouteTable};

use http::request::Parts;

use hyper::body::Body as HyperBody;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

use json;
use json::JsonValue;

type Task = Pin<Box<dyn Sync + Send + Future<Output = ()>>>;


/// A RequestState tracks the state of an incoming HTTP request across its entire lifetime.
pub struct RequestState<'request> {

    pub table: &'request RouteTable,
    pub route: &'request Route,
    pub parts: Parts,

    pub variables: HashMap<String, String>,
    pub headers: HashMap<String, String>,

    // pub object: JsonValue,
    pub object: json::object::Object,

    pub body: HyperBody,
    pub mime: Option<String>,

    pub futures: Vec<Task>,

    pub status: u16,
}

impl<'request> RequestState<'request> {

    // pub fn new(route: &'request Route, request: &'request Request<hyper::Body>, table: &'request RouteTable) -> Self  {
    pub fn new(route: &'request Route, request: Request<hyper::Body>, table: &'request RouteTable) -> Self  {

        // populate variables with key value pairs in the query string
        let mut variables = HashMap::new();
        let mut url_string = "http://0.0.0.0".to_string();
        url_string.push_str(&request.uri().to_string());
        let url = Url::parse(&url_string).expect("Could not parse url");

        for (key, value) in url.query_pairs() {
            variables.insert(format!("query.{}", key), value.to_string());
        }

        let (parts, body) = request.into_parts();

        Self {
            table,
            parts,
            route,
            variables,

            // object: JsonValue::new_object(),
            object: json::object::Object::new(),
            body,

            headers: HashMap::new(),
            mime: None,
            status: 200,
            futures: vec![],
        }
    }

    pub fn register_task<T>(&mut self, task: T)
    where T: Future<Output = ()> + Sync + Send + 'static {
        self.futures.push(Box::pin(task));
    }

    pub async fn wait(&mut self) {
        let futures = std::mem::take(&mut self.futures);
        futures_util::future::join_all(futures).await;
    } 

    pub fn set_variable(&mut self, key: &str, value: &str) {
        self.object.insert(key, JsonValue::String(value.to_owned()));
        // self.variables.insert(key.to_string(), value.to_string());
    }

    // pub fn get_variable(&'request self, name: &str) -> Option<&'request str> {
    pub fn get_variable(&'request self, name: &str) -> Option<String> {

        let value = self.object.get(name)?;
        return Some(value.dump())

        // TODO: add back a way to access the body as a variable.
        // Because the body is a stream now this gets more complicated since you need to await the
        // end of the stream in order to get all its data
        // if name == "body" {
        //     self.wait().await;
        //     let body = std::mem::take(&mut self.body);
        //     let bytes = hyper::body::to_bytes(body).await.unwrap();
        //     return Some(std::str::from_utf8(&bytes).unwrap());
        // }

        // self.variables.get(name).and_then(|val| Some(val.as_str())) //.unwrap_or("")
    }

    // Automatically detect the mime type of the response
    pub fn set_mime_type(&mut self) {
        // println!("{:?}", &self.mime);
        // println!("{:?}", tree_magic::from_u8(&self.body.data()));

        // match &self.mime {
        //     Some(mime_type) => self.headers.insert("Content-Type".to_string(), mime_type.to_string()),
        //     None => self.headers.insert("Content-Type".to_string(), tree_magic::from_u8(&self.body.data())),
        //     // None => Some("text/plain".to_string()),
        //     // None => todo!(),
        // };
    }

}

impl Into<Response<hyper::Body>> for RequestState<'_> {
    fn into(mut self) -> Response<hyper::Body> {
        let mut out = hyper::Response::builder().status(self.status);

        // self.set_mime_type();
        if let Some(mime) = self.mime {
            out = out.header("Content-Type", mime);
        }

        for (key, value) in self.headers.iter() {
            // out.headers_mut().insert(hyper::header::HeaderName.from_lowercase())
            out = out.header(key.as_str(), value.as_str());
        }

        out.body((self.body)).unwrap()
    }
}

use std::fmt::Debug;
use std::fmt;
impl<'request> Debug for RequestState<'request> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("State")
            .field("status", &self.status)
            // .field("request_body", &self.request.body())
            // .field("body", &format!("{:?}", self.body))
            .field("headers", &self.headers)
            // .field("vars", &self.variables)
            .field("vars", &self.object.pretty(2))
            .finish()
        
    }
}
