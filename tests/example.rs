use serde::Deserialize;
use serde_shon::from_args;
use std::env;

#[derive(Deserialize, Debug)]
struct Data {
    field: Option<String>,
}

// Usage: `cargo test --test example -- [ --field hello ]`
fn main() {
    let d: Data = from_args(env::args()).unwrap_or(Data { field: None });
    dbg!(d.field);
}
