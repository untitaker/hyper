use std::fmt;
use std::io::Cursor;
use std::sync::mpsc;

use tick::{self, Protocol};

use http::{self, Incoming, Parse};
use net::Fresh;

const MAX_BUFFER_SIZE: usize = 8192 + 4096 * 100;

pub struct Conn<H: Handler> {
    transfer: tick::Transfer,
    state: State,
    handler: H,
}

impl<H: Handler> Conn<H> {
    pub fn new(transfer: tick::Transfer, handler: H) -> Conn<H> {
        Conn {
            transfer: transfer,
            state: State::Parsing(Vec::with_capacity(4096)),
            handler: handler,
        }
    }
}

impl<H: Handler> Protocol for Conn<H> {
    fn on_data(&mut self, data: &[u8]) {
        let action = match self.state {
            State::Parsing(ref mut buf) => {
                buf.extend(data);
                match http::parse::<H::Incoming, _>(buf) {
                    Ok(Some((incoming, len))) => {
                        trace!("parsed {} bytes out of {}", len, buf.len());
                        let (tx, rx) = mpsc::channel();
                        self.handler.on_incoming(
                            incoming,
                            http::Stream::new(
                                tx,
                                self.transfer.clone(),
                                (&buf[len..]).to_vec() // TODO: ouch
                            ),
                            http::h1::transfer(self.transfer.clone())
                        );
                        Action::State(State::Http1(StreamRx {
                            rx: rx,
                            state: http::StreamState::Paused,
                        })) //(h1::conn())

                    },
                    Ok(None) => {
                        if buf.len() >= MAX_BUFFER_SIZE {
                            //TODO: Handler.on_too_large_error()
                            debug!("MAX_BUFFER_SIZE reached, closing");
                            self.transfer.close();
                            Action::State(State::Closed)
                        } else {
                            Action::Nothing
                        }
                    },
                    Err(e) => {
                        let h2_init = b"PRI * HTTP/2";
                        if data.starts_with(h2_init) {
                            trace!("HTTP/2 request!");
                            //TODO: self.state = State::Http2(h2::conn());
                            self.transfer.close();
                            Action::State(State::Closed)
                        } else {
                            //TODO: match on error to send proper response
                            //TODO: have Handler.on_parse_error() or something
                            self.transfer.close();
                            Action::State(State::Closed)
                        }
                    }
                }
            },
            State::Http1(ref mut stream) => { //(ref mut conn) => {
                match stream.state() {
                    Some(&mut http::StreamState::Reading(ref mut r)) => {
                        r.on_data(data);
                        Action::Nothing
                    }
                    Some(&mut http::StreamState::Paused) => {
                        error!("on_data State::Http1::Paused");
                        Action::Nothing
                    }
                    None => {
                        // reader stopped caring?
                        trace!("on_data State::Http1::Dropped");
                        Action::OnData(State::Parsing(Vec::with_capacity(4096)))
                    }
                }
                //conn.on_data(data);
            },
            /*
            State::Http2(ref mut conn) => {
                conn.on_data(data);
            }
            */
            State::Closed => {
                error!("Closed on_data");
                Action::Nothing
            }

        };

        match action {
            Action::State(state) => self.state = state,
            Action::OnData(state) => {
                self.state = state;
                self.on_data(data);
            }
            Action::Nothing => (),
        }
    }

    fn on_eof(&mut self) {
        trace!("unhandled eof");
    }

    fn on_end(&mut self, err: Option<::tick::Error>) {
        trace!("unhandled end");
        if let Some(err) = err {
            error!("on_end err = {:?}", err);
        }
    }
}

enum Action {
    State(State),
    OnData(State),
    Nothing
}

pub trait Handler {
    type Incoming: Parse;
    type Outgoing;
    //fn on_outgoing(&mut self, transfer: http::Transfer<Self::Outgoing, Fresh>);
    fn on_incoming(&mut self,
                   incoming: Incoming<<Self::Incoming as Parse>::Subject>,
                   stream: http::Stream,
                   transfer: http::Transfer<Self::Outgoing, Fresh>);
}

enum State {
    Parsing(Vec<u8>),
    Http1(StreamRx), //(h1::Conn),
    //Http2,
    Closed,
}


struct StreamRx {
    rx: mpsc::Receiver<http::StreamState>,
    state: http::StreamState,
}

impl StreamRx {
    fn state(&mut self) -> Option<&mut http::StreamState> {
        loop {
            match self.rx.try_recv() {
                Ok(s) => self.state = s,
                Err(mpsc::TryRecvError::Empty) => return Some(&mut self.state),
                Err(mpsc::TryRecvError::Disconnected) => return None
            }
        }
    }
}
