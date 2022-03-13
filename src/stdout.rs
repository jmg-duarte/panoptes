use std::io::{Stdout, Write, Result};

const CLEAR_SCREEN: &[u8] = "\u{001b}c".as_bytes();

pub trait StdoutExt {
    fn clear_screen(&mut self) -> Result<()>;
}

impl StdoutExt for Stdout {
    fn clear_screen(&mut self) -> Result<()> {
        self.write_all(CLEAR_SCREEN)
    }
}