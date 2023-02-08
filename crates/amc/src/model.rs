use crate::{
    application::{
        server::{Server, SyncMethod},
        Application,
    },
    client::Client,
    driver::Drive,
    global::{GlobalActor, GlobalMsg},
    properties,
};
use clap::Args;
use stateright::{
    actor::{model_peers, ActorModel, Envelope, Id, Network},
    Property,
};
use std::fmt::Debug;
use std::hash::Hash;

/// Builder of a model.
pub trait ModelBuilder: Debug {
    /// The application being modeled.
    type App: Application + 'static;

    /// The type of the client in the application.
    type Driver: Drive<Self::App> + 'static;

    /// The type of config for the model.
    type Config: Debug + 'static;

    /// The type of history for the model.
    type History: Clone + Debug + Hash;

    /// Generate an application instance.
    fn application(&self, application: usize) -> Self::App;

    /// Generate some drivers for the given application.
    fn drivers(&self, application: usize) -> Vec<Self::Driver>;

    /// Generate the config for the model.
    fn config(&self, model_opts: &ModelOpts) -> Self::Config;

    /// Generate the default history object.
    fn history(&self) -> Self::History;

    /// Generate the properties to be added to the model.
    fn properties(
        &self,
    ) -> Vec<Property<ActorModel<GlobalActor<Self::App, Self::Driver>, Self::Config, Self::History>>>;

    /// Record an input to the application.
    fn record_input(
        &self,
    ) -> fn(
        cfg: &Self::Config,
        history: &Self::History,
        message: Envelope<&GlobalMsg<Self::App>>,
    ) -> Option<Self::History> {
        |_, _, _| None
    }

    /// Record an output from the application.
    fn record_output(
        &self,
    ) -> fn(
        cfg: &Self::Config,
        history: &Self::History,
        message: Envelope<&GlobalMsg<Self::App>>,
    ) -> Option<Self::History> {
        |_, _, _| None
    }
}

/// Options for running a model.
#[derive(Args, Debug)]
pub struct ModelOpts {
    /// Number of servers to run.
    #[clap(long, short, global = true, default_value = "2")]
    pub servers: usize,

    /// Method to sync changes between servers.
    #[clap(long, global = true, default_value = "changes")]
    pub sync_method: SyncMethod,

    /// Whether to perform server restarts.
    #[clap(long, global = true)]
    pub restarts: bool,

    /// Enable checking documents are in sync and don't have any other messages.
    #[clap(long, global = true)]
    pub in_sync_check: bool,

    /// Enable checking documents can be saved and loaded and they remain the same.
    #[clap(long, global = true)]
    pub save_load_check: bool,

    /// Enable checking historical document queries return the same document as latest queries
    /// would.
    #[clap(long, global = true)]
    pub historical_check: bool,

    /// Enable checking documents don't panic.
    #[clap(long, global = true)]
    pub error_free_check: bool,
}

impl ModelOpts {
    /// Create a model to use for checking.
    ///
    /// Intended for use in tests.
    pub fn to_model<M: ModelBuilder>(
        &self,
        model_builder: &M,
    ) -> ActorModel<GlobalActor<M::App, M::Driver>, M::Config, M::History> {
        let config = model_builder.config(self);
        println!("Built config: {:?}", config);
        let history = model_builder.history();
        println!("Built history: {:?}", history);
        let mut model = ActorModel::new(config, history);

        // add servers
        for i in 0..self.servers {
            model = model.actor(GlobalActor::Server(Server {
                peers: model_peers(i, self.servers),
                sync_method: self.sync_method,
                restarts: self.restarts,
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
        let record_request = model_builder.record_input();
        let record_response = model_builder.record_output();
        model
            .record_msg_in(record_request)
            .record_msg_out(record_response)
            .init_network(Network::new_ordered(vec![]))
    }
}
