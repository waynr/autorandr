use clap;

use anyhow::{anyhow, Result};

use autorandr::{Config, Manager};

fn list(mgr: Manager, _: &clap::ArgMatches) -> Result<()> {
    mgr.list();
    Ok(())
}

fn reconcile(mgr: Manager, _: &clap::ArgMatches) -> Result<()> {
    mgr.reconcile()?;
    Ok(())
}

fn cli() -> Result<clap::Command> {
    Ok(clap::Command::new("autorandr")
        .author("wayne warren")
        .version("0.0.1")
        .about("xrandr, automatically")
        .args([clap::Arg::new("verbose")
            .short('v')
            .help("verbosity")
            .action(clap::ArgAction::Count)])
        .subcommands(vec![
            clap::Command::new("list").about("list active, connected, and disconnected outputs"),
            clap::Command::new("reconcile").about("automatically choose from available profiles based on avaliable monitors"),
        ]))
}

fn main() -> Result<()> {
    let cmd = &mut cli()?;
    let matches = cmd.get_matches_mut();
    let mut logger_builder = &mut pretty_env_logger::formatted_builder();

    let level = match matches.get_one::<u8>("verbose").copied() {
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

    match matches.subcommand() {
        Some(("list", subm)) => {
            list(mgr, subm)
        }
        Some(("reconcile", subm)) => {
            reconcile(mgr, subm)
        }
        Some((c, _)) => {
            println!("{}", cmd.render_usage());
            println!("{}", cmd.render_long_help());
            Err(anyhow!("invalid subcommand {}", c))
        }
        None => {
            println!("{}", cmd.render_usage());
            println!("{}", cmd.render_long_help());
            Err(anyhow!("missing subcommand"))
        }
    }
}
