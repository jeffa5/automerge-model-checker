/// amc-todo shows how to implement the application side and client side with a concrete example
///
use crate::apphandle::AppHandle;
use crate::report::Reporter;
use clap::Parser;
use model::History;
use stateright::actor::ActorModel;
use stateright::Checker;
use stateright::CheckerBuilder;
use stateright::Model;

mod app;
mod apphandle;
mod model;
mod report;
mod trigger;

#[derive(Parser, Debug)]
struct Opts {
    #[clap(subcommand)]
    command: SubCmd,

    #[clap(long, short, global = true, default_value = "2")]
    servers: usize,

    #[clap(long, global = true)]
    message_acks: bool,

    #[clap(long, arg_enum, global = true, default_value = "changes")]
    sync_method: SyncMethod,

    #[clap(long, default_value = "8080")]
    port: u16,

    /// Whether to use random ids for todo creation.
    #[clap(long, global = true)]
    random_ids: bool,
}

/// Methods for syncing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ArgEnum)]
pub enum SyncMethod {
    Changes,
    Messages,
    SaveLoad,
}

#[derive(clap::Subcommand, Debug)]
enum SubCmd {
    Serve,
    CheckDfs,
    CheckBfs,
}

fn main() {
    let opts = Opts::parse();

    let model = model::Builder {
        servers: opts.servers,
        sync_method: match opts.sync_method {
            SyncMethod::SaveLoad => amc_core::SyncMethod::SaveLoad,
            SyncMethod::Changes => amc_core::SyncMethod::Changes,
            SyncMethod::Messages => amc_core::SyncMethod::Messages,
        },
        message_acks: opts.message_acks,
        app: AppHandle {
            random_ids: opts.random_ids,
        },
    }
    .into_actor_model()
    .checker()
    .threads(num_cpus::get());
    run(opts, model)
}

fn run(opts: Opts, model: CheckerBuilder<ActorModel<crate::model::Actor, model::Config, History>>) {
    println!("Running with config {:?}", opts);
    match opts.command {
        SubCmd::Serve => {
            println!("Serving web ui on http://127.0.0.1:{}", opts.port);
            model.serve(("127.0.0.1", opts.port));
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
