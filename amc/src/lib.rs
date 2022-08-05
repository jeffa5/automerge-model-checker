pub mod app;
pub mod client;
pub mod model;
pub mod trigger;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ArgEnum)]
pub enum ObjectType {
    Map,
    List,
}
