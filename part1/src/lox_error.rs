pub trait LoxError {
    fn error(&mut self, line: i32, message: &str);

    fn report(&mut self, line: i32, wh: &str, message: &str);

    fn has_error(&self) -> bool;
}
