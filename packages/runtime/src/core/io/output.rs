use std::io::Write;

use crate::Value;

pub trait OutputHandler: Send {
    fn print(&mut self, content: Value);
}

pub struct ConsoleOutputHandler;
impl OutputHandler for ConsoleOutputHandler {
    fn print(&mut self, content: Value) {
        print!("{}", content);
        std::io::stdout().flush().ok();
    }
}
