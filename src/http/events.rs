pub trait Read {
    fn on_data(&mut self, data: &[u8]);
    fn on_error(&mut self, err: ::Error);
    fn on_eof(&mut self);
}
