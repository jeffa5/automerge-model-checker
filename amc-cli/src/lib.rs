use amc_core::{model, Application, GlobalActor, GlobalMsg, Reporter, Server, Trigger};
use clap::Parser;
use stateright::{
    actor::{model_peers, Actor, ActorModel, Envelope, Network},
    Checker, Model, Property,
};
use std::fmt::Debug;
use std::hash::Hash;

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

impl Opts {
    fn actor_model<C: Cli>(
        &self, c: &C,
    ) -> ActorModel<GlobalActor<C::Client, C::App>, C::Config, C::History> {
        let mut model = ActorModel::new(c.config(self), c.history());

        // add servers
        for i in 0..self.servers {
            model = model.actor(GlobalActor::Server(Server {
                peers: model_peers(i, self.servers),
                sync_method: self.sync_method,
                app: c.application(i),
            }))
        }

        // add triggers
        for i in 0..self.servers {
            for client in c.clients(i) {
                model = model.actor(GlobalActor::Trigger(client));
            }
        }

        model = model::with_default_properties(model);
        for property in c.properties() {
            model = model.property(property.expectation, property.name, property.condition);
        }
        let record_request = c.record_request();
        let record_response = c.record_response();
        model
            .record_msg_in(record_request)
            .record_msg_out(record_response)
            .init_network(Network::new_ordered(vec![]))
    }

    pub fn run<C:Cli>(self, c:C)
        where
            C::Config: Send,
            C::Config: Sync,
            <C::Client as Actor>::State: Sync,
            <C::Client as Actor>::State: Send,
            C::History: Send + Sync + 'static,
    {

        println!("{:?}", self);
        let model = self.actor_model(&c).checker().threads(num_cpus::get());
        println!("Running");

        match self.command {
            SubCmd::Serve => {
                println!("Serving web ui on http://127.0.0.1:{}", self.port);
                model.serve(("127.0.0.1", self.port));
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

pub trait Cli: Debug {
    type App: Application + 'static;
    type Client: Trigger<Self::App> + 'static;
    type Config: 'static;
    type History: Clone + Debug + Hash;

    fn application(&self, server: usize) -> Self::App;
    fn clients(&self, server: usize) -> Vec<Self::Client>;

    fn config(&self, cli_opts: &Opts) -> Self::Config;
    fn history(&self) -> Self::History;

    fn properties(
        &self,
    ) -> Vec<Property<ActorModel<GlobalActor<Self::Client, Self::App>, Self::Config, Self::History>>>;

    fn record_request(
        &self,
    ) -> fn(
        cfg: &Self::Config,
        history: &Self::History,
        message: Envelope<&GlobalMsg<Self::App>>,
    ) -> Option<Self::History> {
        |_, _, _| None
    }
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
