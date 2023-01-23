use crate::{
    application::{server::SyncMethod, Application},
    driver::Drive,
    global::{GlobalActor, GlobalMsg},
};
use clap::Args;
use stateright::{
    actor::{ActorModel, Envelope},
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
