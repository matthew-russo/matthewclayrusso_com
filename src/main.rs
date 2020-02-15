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

const BASE: &str = "/Users/matthewrusso/projects/rust/matthewclayrusso_com";
const RESOURCES: &str = "resources";
const BLOG: &str = "blog";

fn main() {
    let addr = "0.0.0.0:4919".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(Server::new())).unwrap();
    server.run().unwrap();
}

struct Server {
    index_page: Vec<u8>,
    blog_page: Vec<u8>,
}

impl Server {
    pub fn new() -> Self {
        let index_page = Self::load_file(format!("{}/html/index.html", RESOURCES)).expect("failed to load index page"); 
        let blog_page = Self::load_file(format!("{}/index.html", BLOG)).expect("failed to load blog page");
        Self {
            index_page,
            blog_page,
        }
    }

    fn load_file(path: String) -> Option<Vec<u8>> {
        let html_file = format!("{}/{}", BASE, path);
        File::open(html_file)
            .ok()
            .and_then(|mut f| {
                let mut contents: Vec<u8> = Vec::new();
                f.read_to_end(&mut contents)
                    .ok()
                    .map(|_| contents)
            })
    }

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

            Self::load_file(format!("{}/{}.html", BLOG, filename))
                .or(Some(Vec::clone(&self.blog_page)))
        } else {
            None
        }
    }

    fn serve_static(&self, req: &Request) -> Option<Vec<u8>> {
        let static_re = Regex::new(r"^/static/((?:[a-zA-Z_-]+/)+[a-zA-Z_-]+\.[a-zA-Z]+)$").unwrap();

        if let Some(caps) = static_re.captures(req.path()) {
            let filename = caps.get(1).unwrap().as_str();
            Self::load_file(format!("{}/{}.html", RESOURCES, filename))
        } else {
            None
        }
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
            Vec::clone(&self.index_page)
        };

        Box::new(futures::future::ok(
            Response::new()
                .with_header(ContentLength(contents.len() as u64))
                .with_body(contents)
        ))
    }
}
