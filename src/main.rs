use clap;

use autorandr::{Config, Manager, Result};

fn list() -> Result<clap::Command> {
    Ok(clap::Command::new("list").about("list profiles and displays"))
}

fn cli() -> Result<clap::ArgMatches> {
    let list = list()?;
    Ok(clap::Command::new("autorandr")
        .author("wayne warren")
        .version("0.0.1")
        .about("xrandr, automatically")
        .args([clap::Arg::new("verbose")
            .short('v')
            .help("verbosity")
            .action(clap::ArgAction::Count)])
        .subcommands(vec![list])
        .get_matches())
}

fn main() -> Result<()> {
    let matches = cli()?;
    let mut logger_builder = &mut pretty_env_logger::formatted_builder();

    let verbosity = matches.get_one::<u8>("verbose").copied();
    let level = match verbosity {
        Some(0) => log::LevelFilter::Info,
        Some(1) => log::LevelFilter::Debug,
        Some(_) => log::LevelFilter::Trace,
        None => log::LevelFilter::Info,
    };

    logger_builder = logger_builder.filter_level(level);
    if level == log::LevelFilter::Info {
        logger_builder = logger_builder.default_format();
        logger_builder = logger_builder.format_module_path(false);
        logger_builder = logger_builder.format_level(false);
        logger_builder = logger_builder.format_timestamp(None);
    }

    logger_builder.try_init()?;
    log::debug!("verbosity set to {0}", level);

    let cfg = Config::load()?;
    let mgr = Manager::from(cfg).detect()?;
    mgr.list();

    Ok(())
}
