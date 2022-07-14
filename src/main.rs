use amc::model;
use amc::register::MyRegisterActor;
use amc::report::Reporter;
use amc::server::SyncMethod;
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
    put_clients: usize,

    #[clap(long, short, global = true, default_value = "2")]
    delete_clients: usize,

    #[clap(long, short, global = true, default_value = "2")]
    insert_clients: usize,

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

#[derive(clap::Subcommand, Debug)]
enum SubCmd {
    Serve,
    CheckDfs,
    CheckBfs,
}

fn main() {
    let opts = Opts::parse();

    let model = model::Builder {
        put_clients: opts.put_clients,
        delete_clients: opts.delete_clients,
        insert_clients: opts.insert_clients,
        servers: opts.servers,
        sync_method: opts.sync_method,
        message_acks: opts.message_acks,
        object_type: opts.object_type,
    }
    .into_actor_model()
    .checker()
    .threads(num_cpus::get());
    run(opts, model)
}

fn run(opts: Opts, model: CheckerBuilder<ActorModel<MyRegisterActor, model::Config>>) {
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
