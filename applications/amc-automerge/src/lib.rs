pub mod app;
pub mod client;
pub mod driver;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ValueEnum)]
pub enum ObjectType {
    Map,
    List,
}
