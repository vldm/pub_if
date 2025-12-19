# pub_if

A Rust procedural macro that conditionally makes struct fields public based on cfg attributes.
`syn`-free implementation using only `proc_macro` APIs.

[![crates.io](https://img.shields.io/crates/v/pub_if.svg)](https://crates.io/crates/pub_if)
[![docs.rs](https://docs.rs/pub_if/badge.svg)](https://docs.rs/pub_if)

## Usage

Add the `#[pub_if(...)]` attribute to a struct to generate two versions:
- One with `#[cfg(...)]` where all fields are public
- One with `#[cfg(not(...))]` where fields retain their original visibility

```rust
use pub_if::pub_if;

#[pub_if(feature = "foo")]
pub struct Struct<F, B> {
    field: F,
    bar: B,
}
```

This expands to:

```rust
#[cfg(feature = "foo")]
pub struct Struct<F, B> {
    pub field: F,
    pub bar: B,
}

#[cfg(not(feature = "foo"))]
pub struct Struct<F, B> {
    field: F,
    bar: B,
}
```

## Features

- Fields already marked `pub` remain public in both versions
- Supports generic types
- Works with any cfg condition (features, target_os, etc.)
- Implemented without the `syn` crate using only `proc_macro` APIs

## Examples

### Mixed visibility

```rust
#[pub_if(feature = "expose_internals")]
pub struct Config {
    private_setting: i32,
    pub public_setting: String,
}
```

When the feature is enabled, both fields are public.
When disabled, only `public_setting` remains public.

## Testing

The project includes compile-time tests using `trybuild` to verify that:
- Fields are public when the cfg condition is enabled
- Fields remain private when the cfg condition is disabled
- Fields already marked `pub` stay public in both cases

```bash
# Run tests without feature (verifies private fields are not accessible)
cargo test

# Run tests with feature (verifies all fields become public)
cargo test --features foo
```
