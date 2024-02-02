# serde_shon

[![Crates.io](https://img.shields.io/crates/v/serde_shon)](https://crates.io/crates/serde_shon)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/chaosteil/serde_shon/ci.yml?branch=main)](https://github.com/chaosteil/serde_shon/actions)

serde_shon is a Rust library for parsing the SHON data format. The definition of
the format is based on the description present in [shon-go](https://github.com/abhinav/shon-go).

This library is intended to be used with [Serde](https://serde.rs/).

## Installation

Include the library as part of the dependencies in `Cargo.toml`:

```toml
[dependencies]
serde_shon = "0.1.0"
```

## Usage

```rs
use serde::Deserialize;
use serde_shon::from_args;
use std::env;

#[derive(Deserialize, Debug)]
struct Data {
    field: Option<String>,
}

fn main() {
    let d: Data = from_args(env::args()).unwrap();
    dbg!(d.field);
}
```

## Features

The serializer supports common Rust data types for serialization and
deserialization, like enums and structs.

The library might currently still have a few bugs and be incomplete in the
implementation. If you find something troubling, either write up an issue or
perhaps even a PR, contributions are always welcome.
