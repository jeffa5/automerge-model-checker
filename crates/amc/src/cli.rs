use clap::Parser;
use stateright::{actor::ActorModel, Checker, Model};
use tracing::{debug, subscriber::set_global_default};
use tracing_subscriber::EnvFilter;

use crate::{
    global::GlobalActor,
    model::{ModelBuilder, ModelOpts},
    report::Reporter,
};

/// How to run the model.
#[derive(clap::Subcommand, Clone, Debug)]
pub enum Runner {
    /// Launch an interactive explorer in the browser.
    Explore {
        /// Port to serve UI on.
        #[clap(long, default_value = "8080")]
        port: u16,

        /// Path to jump to in explorer.
        #[clap()]
        path: Option<String>,
    },
    /// Launch a checker using depth-first search.
    CheckDfs,
    /// Launch a checker using depth-first search, iterating over progressively larger depths.
    CheckIterative,
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
    pub model_opts: ModelOpts,

    #[clap(long, global = true, default_value = "1000")]
    /// Max depth to search to.
    pub max_depth: usize,
}

impl RunArgs {
    fn build_checker<M: ModelBuilder>(
        &self,
        model: &ActorModel<GlobalActor<M::App, M::Driver>, M::Config, M::History>,
    ) -> stateright::CheckerBuilder<
        ActorModel<
            GlobalActor<<M as ModelBuilder>::App, <M as ModelBuilder>::Driver>,
            <M as ModelBuilder>::Config,
            <M as ModelBuilder>::History,
        >,
    >
    where
        M::Config: Send,
        M::Config: Sync + Clone,
        M::History: Send + Sync + 'static,
    {
        let mut checker = model.clone().checker();
        checker = checker.target_max_depth(self.max_depth);
        checker = checker.threads(std::thread::available_parallelism().unwrap().get());
        checker
    }

    /// Run an application.
    pub fn run<M: ModelBuilder>(self, model_builder: M)
    where
        M::Config: Send,
        M::Config: Sync + Clone,
        M::History: Send + Sync + 'static,
    {
        let collector = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_env_filter(EnvFilter::from_default_env())
            .finish();
        set_global_default(collector).unwrap();

        println!("{:?}", self);
        println!("{:?}", model_builder);
        let model = self.model_opts.to_model(&model_builder);

        let checker = self.build_checker::<M>(&model);

        match self.command {
            Runner::Explore { port, path } => {
                let path = path.map(|p| format!("/#/steps/{}", p)).unwrap_or_default();
                println!("Serving web ui on http://127.0.0.1:{}{}", port, path);
                checker.serve(("127.0.0.1", port));
            }
            Runner::CheckDfs => {
                checker.spawn_dfs().report(&mut Reporter::default()).join();
            }
            Runner::CheckIterative => {
                let limit = self.max_depth;
                for max_depth in 1..=limit {
                    println!("Checking with max depth {}", max_depth);
                    let mut checker = self.build_checker::<M>(&model);
                    checker = checker.target_max_depth(max_depth);
                    let checker = checker.spawn_dfs().report(&mut Reporter::default()).join();

                    let finished = checker.model().properties().iter().all(|property| {
                        let discovery = checker.discovery(property.name);
                        debug!(?property.expectation, ?property.name, ?discovery, "checking for discovery");
                        match property.expectation {
                            stateright::Expectation::Always => {
                                if discovery.is_some() {
                                    return true
                                }
                            }
                            stateright::Expectation::Eventually => {
                                if discovery.is_some() {
                                    return true
                                }
                            }
                            stateright::Expectation::Sometimes => {
                                if discovery.is_none() {
                                    return true
                                }
                            }
                        }
                        false
                    });
                    if finished {
                        break;
                    }
                }
            }
            Runner::CheckBfs => {
                checker.spawn_bfs().report(&mut Reporter::default()).join();
            }
        }
    }
}
