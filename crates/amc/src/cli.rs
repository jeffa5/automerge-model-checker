use clap::Parser;
use stateright::{Checker, Model};
use tracing::subscriber::set_global_default;
use tracing_subscriber::EnvFilter;

use crate::{
    model::{ModelBuilder, ModelOpts},
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
    pub model_opts: ModelOpts,

    #[clap(long, global = true)]
    /// Max depth to search to.
    pub max_depth: Option<usize>,
}

impl RunArgs {
    /// Run an application.
    pub fn run<M: ModelBuilder>(self, model_builder: M)
    where
        M::Config: Send,
        M::Config: Sync,
        M::History: Send + Sync + 'static,
    {
        let collector = tracing_subscriber::fmt().with_ansi(false).with_env_filter(EnvFilter::from_default_env()).finish();
        set_global_default(collector).unwrap();

        println!("{:?}", self);
        println!("{:?}", model_builder);
        let model = self.model_opts.to_model(&model_builder);
        let mut checker = model.checker();
        if let Some(max_depth) = self.max_depth {
            checker = checker.target_max_depth(max_depth);
        }
        checker = checker.threads(std::thread::available_parallelism().unwrap().get());

        match self.command {
            Runner::Explore { port } => {
                println!("Serving web ui on http://127.0.0.1:{}", port);
                checker.serve(("127.0.0.1", port));
            }
            Runner::CheckDfs => {
                checker.spawn_dfs().report(&mut Reporter::default()).join();
            }
            Runner::CheckBfs => {
                checker.spawn_bfs().report(&mut Reporter::default()).join();
            }
        }
    }
}
