use murex_common::{Key, Value};

#[derive(Debug)]
pub enum Command {
    Ping(Option<Value>),
    Get(Key),
    Set(Key, Value),
    Delete(Key),
    Help,
}

#[derive(Debug)]
pub enum Response {
    Ok(Option<Value>),
    Error(String),
    NotFound,
    Help(String),
}
