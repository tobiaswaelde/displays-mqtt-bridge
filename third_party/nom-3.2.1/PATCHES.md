# Local patch

This directory is an unmodified copy of `nom` 3.2.1 except for mechanical
Rust-compatibility fixes. They remove deprecated macro-expansion syntax,
obsolete range patterns and trait-object syntax, invalid inline attributes,
and redundant unsafe/transmute code.

`ddc-hi` 0.4.1 requires the `nom` 3.x API through `edid`, `mccs-caps`, and
`mccs-db`; replacing it with a newer major version is not API-compatible. The
root `Cargo.toml` selects this local patch through `[patch.crates-io]`.
