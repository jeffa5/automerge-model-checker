use amc_core::{model, Application, GlobalActor, Reporter, Server, Trigger};
use stateright::{
    actor::{model_peers, Actor, ActorModel, Network},
    Checker, Model, Property,
};
use std::{fmt::Debug, marker::Send};
use clap::Parser;

/// Options for the main running.
#[derive(Parser, Debug)]
pub struct Opts {
    #[clap(subcommand)]
    pub command: SubCmd,

    /// Number of servers to run.
    #[clap(long, short, global = true, default_value = "2")]
    pub servers: usize,

    /// Method to sync changes between servers.
    #[clap(long, global = true, default_value = "changes")]
    pub sync_method: amc_core::SyncMethod,

    /// Port to serve UI on.
    #[clap(long, default_value = "8080")]
    pub port: u16,
}

#[derive(clap::Subcommand, Copy, Clone, Debug)]
pub enum SubCmd {
    Serve,
    CheckDfs,
    CheckBfs,
}

pub struct Builder<A, C, T> {
    pub servers: usize,
    pub sync_method: amc_core::SyncMethod,
    pub app: A,
    pub config: C,
    pub trigger: T,
}

pub trait Cli: Debug {
    type App: Application + 'static;
    type Client: Trigger<Self::App> + 'static;
    type Config: 'static;

    fn application(&mut self, server: usize) -> Self::App;
    fn clients(&mut self, server: usize) -> Vec<Self::Client>;

    fn config(&self) -> Self::Config;
    fn servers(&self) -> usize;
    fn sync_method(&self) -> amc_core::SyncMethod;

    fn properties(
        &self,
    ) -> Vec<Property<ActorModel<GlobalActor<Self::Client, Self::App>, Self::Config>>>;

    fn actor_model(
        &mut self,
    ) -> ActorModel<GlobalActor<Self::Client, Self::App>, Self::Config, ()> {
        let mut model = ActorModel::new(self.config(), ());

        // add servers
        for i in 0..self.servers() {
            model = model.actor(GlobalActor::Server(Server {
                peers: model_peers(i, self.servers()),
                sync_method: self.sync_method(),
                app: self.application(i),
            }))
        }

        // add triggers
        for i in 0..self.servers() {
            for client in self.clients(i) {
                model = model.actor(GlobalActor::Trigger(client));
            }
        }

        model = model::with_default_properties(model);
        for property in self.properties() {
            model = model.property(property.expectation, property.name, property.condition);
        }
        model.init_network(Network::new_ordered(vec![]))
    }

    fn command(&self) -> SubCmd;
    fn port(&self) -> u16;

    fn run(&mut self)
    where
        <Self as Cli>::Config: Send,
        <Self as Cli>::Config: Sync,
        <<Self as Cli>::Client as Actor>::State: Sync,
        <<Self as Cli>::Client as Actor>::State: Send,
    {
        println!("{:?}", self);
        let model = self.actor_model().checker().threads(num_cpus::get());
        println!("Running");

        match self.command() {
            SubCmd::Serve => {
                println!("Serving web ui on http://127.0.0.1:{}", self.port());
                model.serve(("127.0.0.1", self.port()));
            }
            SubCmd::CheckDfs => {
                model
                    .spawn_dfs()
                    .report(&mut Reporter::default())
                    .join()
                    .assert_properties();
            }
            SubCmd::CheckBfs => {
                model
                    .spawn_bfs()
                    .report(&mut Reporter::default())
                    .join()
                    .assert_properties();
            }
        }
    }
}
