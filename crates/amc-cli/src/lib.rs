#![deny(missing_docs)]

//! Utilities for building models and appropriate CLIs for Automerge Model Checker applications.

use amc::{
    application::{
        server::{Server, SyncMethod},
        Application,
    },
    driver::{client::Client, Drive},
    global::{GlobalActor, GlobalMsg},
    properties,
};
use clap::Parser;
use stateright::{
    actor::{model_peers, ActorModel, Envelope, Id, Network},
    Checker, Model, Property,
};
use std::fmt::Debug;
use std::hash::Hash;

mod report;

pub use report::Reporter;

/// Options for the main running.
#[derive(Parser, Debug)]
pub struct Opts {
    /// How to run the model.
    #[clap(subcommand)]
    pub command: Runner,

    /// Number of servers to run.
    #[clap(long, short, global = true, default_value = "2")]
    pub servers: usize,

    /// Method to sync changes between servers.
    #[clap(long, global = true, default_value = "changes")]
    pub sync_method: SyncMethod,

    /// Port to serve UI on.
    #[clap(long, global = true, default_value = "8080")]
    pub port: u16,

    /// Enable checking documents are in the same state during checking of the document.
    #[clap(long, global = true)]
    pub same_state_check: bool,

    /// Enable checking documents are in sync and don't have any other messages.
    #[clap(long, global = true)]
    pub in_sync_check: bool,

    /// Enable checking documents can be saved and loaded and they remain the same.
    #[clap(long, global = true)]
    pub save_load_check: bool,

    /// Enable checking documents don't panic.
    #[clap(long, global = true)]
    pub error_free_check: bool,
}

/// How to run the model.
#[derive(clap::Subcommand, Copy, Clone, Debug)]
pub enum Runner {
    /// Launch an interactive explorer in the browser.
    Explore,
    /// Launch a checker using depth-first search.
    CheckDfs,
    /// Launch a checker using breadth-first search.
    CheckBfs,
}

impl Opts {
    fn actor_model<M: ModelBuilder>(
        &self,
        model_builder: &M,
    ) -> ActorModel<GlobalActor<M::App, M::Driver>, M::Config, M::History> {
        let mut model = ActorModel::new(model_builder.config(self), model_builder.history());

        // add servers
        for i in 0..self.servers {
            model = model.actor(GlobalActor::Server(Server {
                peers: model_peers(i, self.servers),
                sync_method: self.sync_method,
                app: model_builder.application(i),
            }))
        }

        // add drivers
        for i in 0..self.servers {
            for driver in model_builder.drivers(i) {
                model = model.actor(GlobalActor::Client(Client {
                    server: Id::from(i),
                    driver,
                    _app: std::marker::PhantomData,
                }));
            }
        }

        if self.same_state_check {
            model = properties::with_same_state_check(model);
        }
        if self.in_sync_check {
            model = properties::with_in_sync_check(model);
        }
        if self.save_load_check {
            model = properties::with_save_load_check(model);
        }
        if self.error_free_check {
            model = properties::with_error_free_check(model);
        }

        for property in model_builder.properties() {
            model = model.property(property.expectation, property.name, property.condition);
        }
        let record_request = model_builder.record_request();
        let record_response = model_builder.record_response();
        model
            .record_msg_in(record_request)
            .record_msg_out(record_response)
            .init_network(Network::new_ordered(vec![]))
    }

    /// Run an application.
    pub fn run<M: ModelBuilder>(self, c: M)
    where
        M::Config: Send,
        M::Config: Sync,
        M::History: Send + Sync + 'static,
    {
        println!("{:?}", self);
        let model = self.actor_model(&c).checker().threads(num_cpus::get());

        match self.command {
            Runner::Explore => {
                println!("Serving web ui on http://127.0.0.1:{}", self.port);
                model.serve(("127.0.0.1", self.port));
            }
            Runner::CheckDfs => {
                model
                    .spawn_dfs()
                    .report(&mut Reporter::default())
                    .join()
                    .assert_properties();
            }
            Runner::CheckBfs => {
                model
                    .spawn_bfs()
                    .report(&mut Reporter::default())
                    .join()
                    .assert_properties();
            }
        }
    }
}

/// Trait to manage an application, building the model before running it.
pub trait ModelBuilder: Debug {
    /// The application being modeled.
    type App: Application + 'static;

    /// The type of the client in the application.
    type Driver: Drive<Self::App> + 'static;

    /// The type of config for the model.
    type Config: 'static;

    /// The type of history for the model.
    type History: Clone + Debug + Hash;

    /// Generate an application instance.
    fn application(&self, application: usize) -> Self::App;

    /// Generate some drivers for the given application.
    fn drivers(&self, application: usize) -> Vec<Self::Driver>;

    /// Generate the config for the model.
    fn config(&self, cli_opts: &Opts) -> Self::Config;

    /// Generate the default history object.
    fn history(&self) -> Self::History;

    /// Generate the properties to be added to the model.
    fn properties(
        &self,
    ) -> Vec<Property<ActorModel<GlobalActor<Self::App, Self::Driver>, Self::Config, Self::History>>>;

    /// Record a request to the application.
    fn record_request(
        &self,
    ) -> fn(
        cfg: &Self::Config,
        history: &Self::History,
        message: Envelope<&GlobalMsg<Self::App>>,
    ) -> Option<Self::History> {
        |_, _, _| None
    }

    /// Record a response from the application.
    fn record_response(
        &self,
    ) -> fn(
        cfg: &Self::Config,
        history: &Self::History,
        message: Envelope<&GlobalMsg<Self::App>>,
    ) -> Option<Self::History> {
        |_, _, _| None
    }
}
