use clap::{Parser, Subcommand};
use std::error::Error;

use pikaxe::SystemInfo;

mod dir2milo;
mod milo2dir;
mod milo2kr;
mod savemilo;
pub use self::dir2milo::*;
pub use self::milo2dir::*;
pub use self::milo2kr::*;
pub use self::savemilo::*;

// From Cargo.toml
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) trait SubApp {
    fn process(&mut self) -> Result<(), Box<dyn Error>>;
}

#[derive(Parser, Debug)]
#[command(name = PKG_NAME, version = VERSION, about = "Use this tool for modding scenes from milo engine based games")]
struct Options {
    #[command(subcommand)]
    commands: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    #[command(name = "dir2milo", about = "Creates milo scene from input directory")]
    Dir2Milo(Dir2MiloApp),
    #[command(name = "milo2dir", about = "Extracts content of milo scene to directory")]
    Milo2Dir(Milo2DirApp),
    #[command(name = "milo2kr", about = "Converts RB milo lipsync to KR rnd")]
    Milo2Kr(Milo2KrApp),
    #[command(name = "savemilo", about = "Save milo")]
    SaveMilo(SaveMiloApp),
}

#[derive(Debug)]
pub struct SceneTool {
    options: Options,
}

impl SceneTool {
    pub fn new() -> SceneTool {
        SceneTool {
            options: Options::parse()
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        match &mut self.options.commands {
            SubCommand::Dir2Milo(app) => app.process(),
            SubCommand::Milo2Dir(app) => app.process(),
            SubCommand::Milo2Kr(app) => app.process(),
            SubCommand::SaveMilo(app) => app.process(),
        }
    }
}

pub trait GameOptions {
    fn get_system_info(&self) -> SystemInfo;
}