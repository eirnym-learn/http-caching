#[derive(Debug)]
// TODO: add custom errors
pub enum Error {
    FIXME,
    CacheOff,
}

pub type Result<T> = std::result::Result<T, Error>;
