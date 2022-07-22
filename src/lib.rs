pub mod app;
pub mod client;
pub mod doc;
pub mod model;
pub mod report;
pub mod trigger;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ArgEnum)]
pub enum ObjectType {
    Map,
    List,
}
