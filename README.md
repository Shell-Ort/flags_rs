# flags_rs: A simple Go-like flag library for Rust

A lightweight library for parsing command-line flags.

Flags with values are passed using the following syntax:

```sh
-flag=value
```

Flags without a value are treated as boolean flags:

```sh
-verbose
```

To print help of the flags (must be used after all the flags are declarated) and be the only flag in args:
```sh
--help  -h
```

## Installation

```toml
[dependencies]
flags_rs = "0.1.0"
```

## Example

```rust
// Returns the parsed value and whether the flag was explicitly provided.
// This allows you to distinguish between a default value and a user-supplied one.

let (value, exists) =
    flags_rs::flag::<i32>(
        "flag", 
        &10, 
        "An example flag."
    ).unwrap();

let (debug, _) = 
    flags_rs::flag::<bool>(
        "debug",
        &false,
        "Enable debug mode."
    ).unwrap();
```