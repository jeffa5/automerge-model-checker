# Automerge Model Checker

An attempt to model check the [Rust implementation](https://github.com/automerge/automerge-rs) of [Automerge](https://automerge.org/) directly, making some simplifying assumptions along the way.

## Status

The [`amc-automerge`](./applications/amc-automerge) crate provides a binary for checking general Automerge checking (exercising the public API a bit).

In the process of implementing this checking I think I've found some potential for checking custom applications/data structures built on-top of Automerge.
This might be able to help designers of data types check their merge behaviour too!

## Current applications

- [Automerge itself](./applications/amc-automerge)
    - `cargo run --release --bin amc-automerge`
- [Todos](./applications/amc-todo)
    - `cargo run --release --bin amc-todo`
- [Counter](./amc/examples/counter.rs)
    - `cargo run --release --example counter`
