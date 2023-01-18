use amc_core::{model, Application, GlobalActor, Reporter, Server, Trigger};
use clap::Parser;
use stateright::{
    actor::{model_peers, Actor, ActorModel, Network},
    Checker, Model,
};

#[derive(clap::Subcommand, Debug)]
enum SubCmd {
    Serve,
    CheckDfs,
    CheckBfs,
}

/// Methods for syncing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ArgEnum)]
pub enum SyncMethod {
    Changes,
    Messages,
    SaveLoad,
}

#[derive(Parser, Debug)]
struct Opts<O: clap::Args> {
    #[clap(subcommand)]
    command: SubCmd,

    #[clap(long, short, global = true, default_value = "2")]
    servers: usize,

    #[clap(long, arg_enum, global = true, default_value = "changes")]
    sync_method: SyncMethod,

    #[clap(long, default_value = "8080")]
    port: u16,

    #[clap(flatten)]
    custom_opts: O,
}

pub struct Builder<A, C, T> {
    pub servers: usize,
    pub sync_method: amc_core::SyncMethod,
    pub app: A,
    pub config: C,
    pub trigger: T,
}

impl<A: Application, C, T: Trigger<A>> Builder<A, C, T> {
    pub fn into_actor_model(self) -> ActorModel<GlobalActor<T, A>, C, ()> {
        let mut model = ActorModel::new(self.config, ());

        // add servers
        for i in 0..self.servers {
            model = model.actor(GlobalActor::Server(Server {
                peers: model_peers(i, self.servers),
                sync_method: self.sync_method,
                app: self.app.clone(),
            }))
        }

        // add triggers
        for _ in 0..self.servers {
            // let i = stateright::actor::Id::from(i);
            model = model.actor(GlobalActor::Trigger(self.trigger.clone()));
        }

        model = model::with_default_properties(model);
        model.init_network(Network::new_ordered(vec![]))
    }
}

pub fn clap<A, C, T, O>(app: A, config: C, trigger: T)
where
    A: Application + 'static,
    C: Send + Sync + 'static,
    T: Trigger<A> + Actor + 'static,
    <T as Actor>::State: Send + Sync,
    O: clap::Args + std::fmt::Debug,
{
    let opts = Opts::<O>::parse();
    println!("Running with config {:?}", opts);

    let model = Builder {
        servers: opts.servers,
        sync_method: match opts.sync_method {
            SyncMethod::SaveLoad => amc_core::SyncMethod::SaveLoad,
            SyncMethod::Changes => amc_core::SyncMethod::Changes,
            SyncMethod::Messages => amc_core::SyncMethod::Messages,
        },
        config,
        trigger,
        app,
    }
    .into_actor_model()
    .checker()
    .threads(num_cpus::get());

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
