use clap::Parser;
use stateright::{
    actor::{model_peers, ActorModel, Id, Network},
    Checker, Model,
};

use crate::{
    application::server::Server,
    client::Client,
    global::GlobalActor,
    model::{ModelBuilder, ModelOpts},
    properties,
    report::Reporter,
};

/// How to run the model.
#[derive(clap::Subcommand, Copy, Clone, Debug)]
pub enum Runner {
    /// Launch an interactive explorer in the browser.
    Explore {
        /// Port to serve UI on.
        #[clap(long, default_value = "8080")]
        port: u16,
    },
    /// Launch a checker using depth-first search.
    CheckDfs,
    /// Launch a checker using breadth-first search.
    CheckBfs,
}

/// Arguments for running a model check.
#[derive(Parser, Debug)]
pub struct RunArgs {
    /// How to run the model.
    #[clap(subcommand)]
    pub command: Runner,

    /// Model opts
    #[clap(flatten)]
    model_opts: ModelOpts,
}

impl RunArgs {
    fn actor_model<M: ModelBuilder>(
        &self,
        model_builder: &M,
    ) -> ActorModel<GlobalActor<M::App, M::Driver>, M::Config, M::History> {
        let config = model_builder.config(&self.model_opts);
        println!("Built config: {:?}", config);
        let history = model_builder.history();
        println!("Built history: {:?}", history);
        let mut model = ActorModel::new(config, history);

        // add servers
        for i in 0..self.model_opts.servers {
            model = model.actor(GlobalActor::Server(Server {
                peers: model_peers(i, self.model_opts.servers),
                sync_method: self.model_opts.sync_method,
                app: model_builder.application(i),
            }))
        }

        // add drivers
        for i in 0..self.model_opts.servers {
            for driver in model_builder.drivers(i) {
                model = model.actor(GlobalActor::Client(Client {
                    server: Id::from(i),
                    driver,
                    _app: std::marker::PhantomData,
                }));
            }
        }

        if self.model_opts.same_state_check {
            model = properties::with_same_state_check(model);
        }
        if self.model_opts.in_sync_check {
            model = properties::with_in_sync_check(model);
        }
        if self.model_opts.save_load_check {
            model = properties::with_save_load_check(model);
        }
        if self.model_opts.error_free_check {
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

    /// Run an application.
    pub fn run<M: ModelBuilder>(self, model_builder: M)
    where
        M::Config: Send,
        M::Config: Sync,
        M::History: Send + Sync + 'static,
    {
        println!("{:?}", self);
        println!("{:?}", model_builder);
        let model = self
            .actor_model(&model_builder)
            .checker()
            .threads(num_cpus::get());

        match self.command {
            Runner::Explore { port } => {
                println!("Serving web ui on http://127.0.0.1:{}", port);
                model.serve(("127.0.0.1", port));
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
