// TODO: add custom errors
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    FIXME,
}

pub type Result<T> = core::result::Result<T, Error>;
