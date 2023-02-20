use amc_moves::Args;
use clap::Parser;
fn main() {
    let Args {
        moves_opts,
        amc_args,
    } = Args::parse();
    amc_args.run(moves_opts);
}
