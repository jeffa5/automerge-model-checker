use amc_automerge::Args;
use clap::Parser;

fn main() {
    let Args {
        automerge_opts,
        amc_args,
    } = Args::parse();
    amc_args.run(automerge_opts);
}
