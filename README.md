# Automerge Model Checker (amc)

[![main docs](https://img.shields.io/badge/docs-main-informational)](http://jeffas.io/automerge-model-checker/doc/amc/)

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

## Design

The Automerge model checker is built on [Stateright](https://github.com/stateright/stateright) for doing the actual model checking.
amc provides convenience wrappers for working with automerge documents as well as a more structured way of building the model to be checked.
This comes down to two main parts: the [`Application`](http://jeffas.io/automerge-model-checker/doc/amc/application/trait.Application.html) and the [`Drive`r](http://jeffas.io/automerge-model-checker/doc/amc/driver/trait.Drive.html).

The application is responsible for implementing the logic of mutating the automerge document.
It takes inputs (think function arguments), executes operations on the document (atomically), and then produces outputs (function return values).

The driver creates the inputs for the application to work with, and can get responses in order to send more inputs too.
Each driver instance should implement a single type of workflow that can be performed.
This gives each application multiple drivers.

When building the model using the [`ModelBuilder`](http://jeffas.io/automerge-model-checker/doc/amc/model/trait.ModelBuilder.html), amc will combine applications with logic to handle syncing with a certain method.
Drivers get combined with a lightweight wrapper to handle communication with the application.
It is at this point developers can specify properties that they want to be evaluated in the model.
For instance, a counter should have the value of the sum of increments and decrements (shown in `properties` of [`CounterOpts`](http://jeffas.io/automerge-model-checker/doc/amc_counter/struct.CounterOpts.html)).
amc provides some common properties and helpers in the [`properties` module](http://jeffas.io/automerge-model-checker/doc/amc/properties/index.html).
