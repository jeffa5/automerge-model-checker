pub mod app;
pub mod client;
pub mod model;
pub mod driver;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, clap::ValueEnum)]
pub enum ObjectType {
    Map,
    List,
}
