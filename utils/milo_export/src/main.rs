mod helpers;

//use pikaxe::io::PathFinder;
use helpers::*;
use std::env;
use std::path::PathBuf;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();

    if args.len() < 2 {
        println!("milo_export.exe [input_game_dir_path] [output_dir_path]");
        return;
    }

    let game_dir = PathBuf::from(&args[0]);
    let output_dir = PathBuf::from(&args[1]);

    let mut analyzer = GameAnalyzer::new(game_dir);
    analyzer.process();
    analyzer.export(&output_dir);
}
