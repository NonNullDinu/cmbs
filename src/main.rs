#[macro_use]
extern crate log;

use std::path::{Path, PathBuf};
use std::process::exit;

use clap::AppSettings;
use clap::Clap;
use leafbuild::handle::{config::Config, Handle};
use leafbuild::interpreter;
use log::LevelFilter;

#[derive(Debug, Clap)]
struct BuildCommand {
    #[clap(short, long = "dir", parse(from_os_str), default_value = ".")]
    directory: PathBuf,
    #[clap(
        short = 'o',
        long = "output-dir",
        parse(from_os_str),
        default_value = "leafbuild-dir"
    )]
    output_directory: PathBuf,
    #[clap(long = "disable-error-cascade")]
    disable_error_cascade: bool,
    #[clap(long = "ci")]
    ci_enabled: bool,

    #[clap(long = "build-failure-signals")]
    build_failure_signals: bool,
}

#[derive(Debug, Clap)]
#[clap(setting(AppSettings::Hidden))]
enum InternalSubcommand {
    #[clap(name = "compilation-failed")]
    CompilationFailed {
        #[clap(long = "exit-code")]
        exit_code: i32,
        #[clap(long = "in")]
        in_: String,
        #[clap(long = "out")]
        out: String,
        #[clap(long = "module-id")]
        mod_id: usize,
    },
    #[clap(name = "link-failed")]
    LinkFailed {
        #[clap(long = "exit-code")]
        exit_code: i32,
        #[clap(long = "in")]
        in_: String,
        #[clap(long = "out")]
        out: String,
        #[clap(long = "module-id")]
        mod_id: usize,
    },
}

#[derive(Debug, Clap)]
enum Subcommand {
    Build {
        #[clap(flatten)]
        build_command: BuildCommand,
    },
    Internal {
        #[clap(flatten)]
        internal_subcommand: InternalSubcommand,
    },
}

#[derive(Debug, Clap)]
#[clap(
    name = "leafbuild",
    about = "Automates C/C++ builds",
    setting = AppSettings::ColoredHelp,
)]
struct Cli {
    #[clap(flatten)]
    subcommand: Subcommand,
}

fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(LevelFilter::Warn)
        .init();
    let cli: Cli = Cli::parse();

    match cli.subcommand {
        Subcommand::Build { build_command } => {
            let _wd = std::env::current_dir().unwrap();
            let proj_path = Path::new(&build_command.directory);
            let ci_enabled = build_command.ci_enabled;
            let config = Config::new(
                !build_command.disable_error_cascade,
                build_command.output_directory,
                ci_enabled || build_command.build_failure_signals,
            );

            let mut handle = Handle::new(config);
            interpreter::start_on(&mut handle, &proj_path.to_path_buf()).unwrap_or_else(|error| {
                error!("An error occurred: {}", error);
            });
        }
        Subcommand::Internal {
            internal_subcommand,
        } => match internal_subcommand {
            InternalSubcommand::CompilationFailed {
                exit_code,
                in_,
                out,
                mod_id,
            } => {
                println!(
                    "exit code = {}, in = '{}', out = '{}', mod_id = {}",
                    exit_code, in_, out, mod_id
                );
                exit(exit_code);
            }
            InternalSubcommand::LinkFailed {
                exit_code,
                in_,
                out,
                mod_id,
            } => {
                println!(
                    "exit code = {}, in = '{}', out = '{}', mod_id = {}",
                    exit_code, in_, out, mod_id
                );
                exit(exit_code);
            }
        },
    }
}
