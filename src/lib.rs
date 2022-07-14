pub mod client;
pub mod doc;
pub mod model;
pub mod register;
pub mod report;
pub mod server;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ArgEnum)]
pub enum ObjectType {
    Map,
    List,
}
