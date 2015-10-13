use std::sync::Arc;

use httparse;

use http;
use net;

use super::{Handler, request, Request, Response};

pub struct Conn<H: Handler> {
    handler: Arc<H>
}

impl<H: Handler> Conn<H> {
    pub fn new(handler: Arc<H>) -> Conn<H> {
        Conn {
            handler: handler,
        }
    }
}

impl<H: Handler> http::Handler for Conn<H> {
    type Incoming = httparse::Request<'static, 'static>;
    type Outgoing = http::Response;

    fn on_incoming(&mut self, incoming: http::IncomingRequest, stream: http::Stream,  transfer: http::Transfer<http::Response, net::Fresh>) {
        let request = request::new(incoming, stream);
        let response = Response::new(transfer);
        self.handler.handle(request, response);
    }
}
