mod cli;
mod configure;
mod download;
mod package;
mod store;

pub use cli::run_cli;
pub use package::Package;
pub use store::PluginStore;
pub type ManagerResult<T> = Result<T, Box<dyn std::error::Error>>;
