# Model checking automerge

An attempt to model check the [Rust implementation](https://github.com/automerge/automerge-rs) of [Automerge](https://github.com/automerge/automerge) directly, making some simplifying assumptions along the way.

## Status

The [`amc`](./amc) crate provides a binary for checking general automerge checking (exercising the public API a bit).

In the process of implementing this checking I think I've found some potential for value in checking custom datastructures built on-top of automerge.
This might be able to help designers of datatypes check their merge behaviour too!
For now there are [`todos`](./amc-todo) and a [counter example](./amc-core/examples/counter.rs).
Both have broken implementations (on purpose!) so try running them to see the failing behaviour.

## Running

### Running amc

```sh
cargo run --bin amc -- serve
```

### Running amc-todo

```sh
cargo run --bin amc-todo -- serve
```

### Running the counter example

```sh
cargo run --example counter -- serve --increments 1 --decrements 0
```

### Without serving the web UI

If you don't want to see the web ui you can do the checking all in the terminal by changing the `serve` command to `check-dfs` or `check-bfs`.
