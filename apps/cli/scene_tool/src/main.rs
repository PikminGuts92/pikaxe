mod apps;
use apps::SceneTool;
use simplelog::*;

#[cfg(debug_assertions)]
const LOG_LEVEL: LevelFilter = LevelFilter::Debug;

#[cfg(not(debug_assertions))]
const LOG_LEVEL: LevelFilter = LevelFilter::Info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_config = ConfigBuilder::new()
        .add_filter_allow_str("pikaxe")
        .add_filter_allow_str("scene_tool")
        .build();

    // Setup logging
    CombinedLogger::init(
        vec![
            TermLogger::new(LOG_LEVEL, log_config, TerminalMode::Mixed, ColorChoice::Auto),
        ]
    )?;

    let mut scene = SceneTool::new();
    scene.run()
}