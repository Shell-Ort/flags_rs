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

To print help of the flags (it must be the only flag in args):
```sh
--help  -h
```

To see the args not flags use non_flags().

## Installation

(See the extra "t")

```toml
[dependencies]
flags_rst = "0.2.0"
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