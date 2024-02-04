# serde_shon

[![Crates.io](https://img.shields.io/crates/v/serde_shon)](https://crates.io/crates/serde_shon)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/chaosteil/serde_shon/ci.yml?branch=main)](https://github.com/chaosteil/serde_shon/actions)

serde_shon is a Rust library for parsing the SHON data format. The definition of
the format is based on the description present in [shon-go](https://github.com/abhinav/shon-go).

This library is intended to be used with [Serde](https://serde.rs/).

## SHON?

SHON (pronounced 'shawn') is short for **Sh**ell **O**bject **N**otation.
It is a notation
for expressing complex objects at the command line.
Because it is intended to be used on the command line,
it aims to reduce extraneous commas and brackets.

All JSON objects can be expressed via SHON,
typically in a format that is easier to specify on the command line.

| JSON                 | SHON                |
|----------------------|---------------------|
| `{"hello": "World"}` | `[ --hello World ]` |
| `["beep", "boop"]`   | `[ beep boop ]`     |
| `[1, 2, 3]`          | `[ 1 2 3 ]`         |
| `[]`                 | `[ ]` or `[]`       |
| `{"a": 10, b: 20}`   | `[ --a 10 --b 20 ]` |
| `{}`                 | `[--]`              |
| `1`                  | `1`                 |
| `-1`                 | `-1`                |
| `1e3`                | `1e3`               |
| `"hello"`            | `hello`             |
| `"hello world"`      | `'hello world'`     |
| `"10"`               | `-- 10`             |
| `"-10"`              | `-- -10`            |
| `"-"`                | `-- -`              |
| `"--"`               | `-- --`             |
| `true`               | `-t`                |
| `false`              | `-f`                |
| `null`               | `-n`                |

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

And you will be able to call your application as such:

```bash
$ ./binary [ --field hello ]
d.field = Some(
    "hello",
)
```

## Features

The serializer supports common Rust data types for serialization and
deserialization, like enums and structs.

The library might currently still have a few bugs and be incomplete in the
implementation. If you find something troubling, either write up an issue or
perhaps even a PR, contributions are always welcome.
