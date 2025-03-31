use std::fmt::Display;

#[derive(Debug)]
pub enum PreCheckErr {
    ZkmainNotExists,
    ZkmainIsNotFunction,
    // ZkmainTypeNotMatch,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum RuntimeErr {}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    PreCheck(PreCheckErr),
    // Runtime(RuntimeErr),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
