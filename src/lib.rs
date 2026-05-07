//! wincd — WSL 下一步到位的 Windows 路径导航工具

pub mod cli;
pub mod clipboard;
pub mod commands;
pub mod converter;
pub mod resolver;
pub mod shell;
pub mod ui;

pub use cli::{run, Cli, Command};
pub use converter::{ConvertError, ConvertResult, Converter, Direction};
pub use shell::Shell;
