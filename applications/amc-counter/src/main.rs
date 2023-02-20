use amc_counter::Args;
use clap::Parser;

fn main() {
    let Args {
        counter_opts,
        amc_args,
    } = Args::parse();
    amc_args.run(counter_opts);
}
