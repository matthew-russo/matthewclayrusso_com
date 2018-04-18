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

fn main() {
    let addr = "127.0.0.1:7777".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(HelloWorld)).unwrap();
    server.run().unwrap();
}

struct HelloWorld;

impl Service for HelloWorld {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let resources = "/Users/matthewrusso/rust/matthewclayrusso_com/resources/";
        let html = "/Users/matthewrusso/rust/matthewclayrusso_com/resources/html/application.html";
        let mut contents: Vec<u8> = Vec::new();

        let re = Regex::new(r"^/static/([a-zA-Z_-]+/[a-zA-Z_-]+\.[a-zA-Z]+)$").unwrap();

        match re.captures(req.path()) {
            Some(caps) => {
                if caps.len() == 2 {
                    let filename = caps.get(1).unwrap().as_str();

                    let composed = format!("{}{}", resources, filename);
                    println!("{}", composed);

                    match File::open(composed) {
                        Ok(mut file) => {
                            file.read_to_end(&mut contents).expect("Something went wrong opening the file");
                        },
                        Err(e) => {
                            println!("ERROR READING FILE: {}", e.description());
                        },
                    }
                } else {
                    println!("not enough captures: {}", caps.len());
                }
            },
            None => {
                let mut f = File::open(html).expect("file not found");

                f.read_to_end(&mut contents)
                    .expect("something went wrong reading the file");
            }
        }

        // We're currently ignoring the Request
        // And returning an 'ok' Future, which means it's ready
        // immediately, and build a Response with the 'PHRASE' body.
        Box::new(futures::future::ok(
            Response::new()
                .with_header(ContentLength(contents.len() as u64))
                .with_body(contents)
        ))
    }
}
