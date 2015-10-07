#![deny(warnings)]
extern crate hyper;
extern crate env_logger;

extern crate eventual;

use hyper::{Get, Post};
use hyper::header::ContentLength;
use hyper::server::{Server, Request, Response};
use hyper::uri::RequestUri::AbsolutePath;


fn echo(req: Request, mut res: Response) {
    match req.uri {
        AbsolutePath(ref path) => match (&req.method, &path[..]) {
            (&Get, "/") | (&Get, "/echo") => {
                res.send(b"Try POST /echo");
                return;
            },
            (&Post, "/echo") => (), // fall through, fighting mutable borrows
            _ => {
                *res.status_mut() = hyper::NotFound;
                return;
            }
        },
        _ => {
            return;
        }
    };

    if let Some(len) = req.headers.get::<ContentLength>() {
        res.headers_mut().set(*len);
    }

    req.stream(Echo(res.start()));
}

struct Echo(hyper::server::Response<hyper::Streaming>);

impl hyper::Read for Echo {
    fn on_data(&mut self, data: &[u8]) {
        println!("data {:?}", ::std::str::from_utf8(data));
        self.0.write(data);
    }

    fn on_error(&mut self, error: hyper::Error) {
        println!("error {:#?}", error);
    }

    fn on_eof(&mut self) {
        println!("eof")
    }
}

fn main() {
    env_logger::init().unwrap();
    let server = Server::http("127.0.0.1:1337").unwrap();
    let _guard = server.handle(echo);
    println!("Listening on http://127.0.0.1:1337");
}
