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
    let addr = "0.0.0.0:7777".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(Server)).unwrap();
    server.run().unwrap();
}

struct Server;

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<dyn Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let resources = "/home/ec2-user/matthewclayrusso_com/resources/";
        let html = "/home/ec2-user/matthewclayrusso_com/resources/html/index.html";
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

        Box::new(futures::future::ok(
            Response::new()
                .with_header(ContentLength(contents.len() as u64))
                .with_body(contents)
        ))
    }
}
