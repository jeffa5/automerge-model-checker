use amc::client;
use amc::client::Client;
use amc::model;
use amc::report::Reporter;
use clap::Parser;
use stateright::actor::ActorModel;
use stateright::Checker;
use stateright::CheckerBuilder;
use stateright::Model;

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

    // What object type to check.
    #[clap(long, arg_enum, global = true, default_value = "map")]
    object_type: amc::ObjectType,

    #[clap(long, default_value = "8080")]
    port: u16,
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
        object_type: opts.object_type,
        app: Client {
            map_single_putter: client::MapSinglePutter,
            list_start_putter: client::ListPutter,
            map_single_deleter: client::MapSingleDeleter,
            list_deleter: client::ListDeleter,
            list_inserter: client::ListInserter,
        },
    }
    .into_actor_model()
    .checker()
    .threads(num_cpus::get());
    run(opts, model)
}

fn run(opts: Opts, model: CheckerBuilder<ActorModel<crate::model::Actor, model::Config>>) {
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
