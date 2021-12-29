use std::fmt::{Debug, Display, Error, Formatter};

use super::PseudoOp;
use crate::tokens::MpsToken;

#[derive(Debug)]
pub struct SyntaxError {
    pub line: usize,
    pub token: MpsToken,
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "SyntaxError (line {}): Unexpected {}",
            &self.line, &self.token
        )
    }
}

impl MpsLanguageError for SyntaxError {
    fn set_line(&mut self, line: usize) {
        self.line = line
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub line: usize,
    pub op: PseudoOp,
    pub msg: String,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} (line {}): {}", &self.msg, &self.line, &self.op)
    }
}

impl MpsLanguageError for RuntimeError {
    fn set_line(&mut self, line: usize) {
        self.line = line
    }
}

pub trait MpsLanguageError: Display + Debug {
    fn set_line(&mut self, line: usize);
}
