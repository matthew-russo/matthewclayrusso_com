extern crate hyper;
extern crate futures;
extern crate regex;

use futures::future::Future;

use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};

use std::fs::File;
use std::io::prelude::*;
use regex::Regex;
use std::error::Error;

const RESOURCES: &str = "/Users/matthewrusso/projects/rust/matthewclayrusso_com/resources";
const BLOG: &str = "/Users/matthewrusso/projects/rust/matthewclayrusso_com/blog";

fn main() {
    let addr = "0.0.0.0:4919".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(Server)).unwrap();
    server.run().unwrap();
}

struct Server;

impl Server {
    fn serve_blog(&self, req: &Request) -> Option<Vec<u8>> {
        let blog_re = Regex::new(r"^/blog(?:/([a-zA-Z_-]+))?$").unwrap();

        if let Some(caps) = blog_re.captures(req.path()) {
            let filename = if caps.len() == 2 {
                if let Some(c) = caps.get(1) {
                    c.as_str()
                } else {
                    "index"
                }
            } else {
                "index"
            };

            return File::open(format!("{}/{}.html", BLOG, filename))
                .ok()
                .map(|mut file| {
                    let mut contents: Vec<u8> = Vec::new();
                    file.read_to_end(&mut contents).expect("Something went wrong opening the file");
                    contents
                });
        } else {
            None
        }
    }

    fn serve_static(&self, req: &Request) -> Option<Vec<u8>> {
        let static_re = Regex::new(r"^/static/((?:[a-zA-Z_-]+/)+[a-zA-Z_-]+\.[a-zA-Z]+)$").unwrap();

        if let Some(caps) = static_re.captures(req.path()) {
            let filename = caps.get(1).unwrap().as_str();

            return File::open(format!("{}/{}", RESOURCES, filename))
                .ok()
                .map(|mut file| {
                    let mut contents: Vec<u8> = Vec::new();
                    file.read_to_end(&mut contents).expect("Something went wrong opening the file");
                    contents
                });
        } else {
            None
        }
    }

    fn serve_index(&self) -> Vec<u8> {
        let html = "/Users/matthewrusso/projects/rust/matthewclayrusso_com/resources/html/index.html";
        let mut f = File::open(html).expect("file not found");
        let mut contents: Vec<u8> = Vec::new();
        f.read_to_end(&mut contents).expect("something went wrong reading the file");
        contents
    }
}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<dyn Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let contents = if let Some(contents) = self.serve_blog(&req) {
            contents
        } else if let Some(contents) = self.serve_static(&req) {
            contents
        } else {
            self.serve_index()
        };

        Box::new(futures::future::ok(
            Response::new()
                .with_header(ContentLength(contents.len() as u64))
                .with_body(contents)
        ))
    }
}
