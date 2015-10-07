use http;

pub struct Conn<H: http::Handler> {
    state: State,
    buf: Vec<u8>,
    handler: Arc<H>,
}

impl Conn {
    pub fn new(data: &[u8], handler: Arc<H>) -> ::Result<Conn> {
        match try!(http::parse::<H::Incoming, _>(data)) {
            Some((incoming, len)) => {
            
            },
            None => ()
        }
        Conn {
            state: Parsing,
            buf: Vec::with_capacity(4096),
        }
    }

    pub fn on_data<H: http::Handler>(&mut self, data: &[u8], handler: &H) {
    
    }
}


enum State {
    Parsing,
    Handling(StreamRx),
    Closed,
}
