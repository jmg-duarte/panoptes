use std::{env::current_dir, io::stdout, path::PathBuf, str::FromStr, thread, time::Duration};

use chrono::prelude::*;
use clap::{Parser, Subcommand};

use panoptes::{db::Database, git::Summarize, stdout::StdoutExt};

fn default_database_path() -> PathBuf {
    let mut default_database_path = PathBuf::from_str(&std::env::var("HOME").unwrap()).unwrap();
    default_database_path.push(".config");
    default_database_path.push("panoptes.sqlite");
    default_database_path
}

#[derive(Debug, Parser)]
struct Cli {
    /// The path to Panoptes' database.
    #[clap(long, parse(from_os_str), default_value_os_t=default_database_path())]
    database: PathBuf,

    #[clap(subcommand)]
    commands: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add repository to the watchlist.
    Add {
        /// Repository directory. If omitted,
        /// the current directory is used.
        #[clap(short, long, parse(from_os_str), default_value_os_t=current_dir().unwrap())]
        directory: PathBuf,

        /// Group name to add the repository to.
        #[clap(short, long)]
        group: Option<String>,
    },

    /// Watch over registered repositories.
    Watch {
        /// The group to watch. If omitted,
        /// all registered repositories will be used.
        #[clap(short, long)]
        group: Option<String>,

        /// The refresh delay (in seconds).
        #[clap(long, default_value_t = 5)]
        delay: u64
    },

    /// Show the current status of the registered repositories.
    Status {
        /// The group to watch. If omitted,
        /// all registered repositories will be used.
        #[clap(short, long)]
        group: Option<String>,
    }
}

fn main() {
    let cli = Cli::parse();

    let db = Database::create(cli.database);

    match cli.commands {
        Commands::Add { directory, group } => {
            // TODO: handle non-repos gracefully
            let _ = git2::Repository::open(&directory).unwrap();
            db.add_repository(directory, group);
        }
        Commands::Watch { group , delay} => {
            let paths = db.get_repositories(group).unwrap();
            let repos: Vec<git2::Repository> = paths
                .into_iter()
                .map(git2::Repository::open)
                .collect::<Result<_, git2::Error>>()
                .unwrap();

            loop {
                stdout().clear_screen().unwrap();
                repos.iter().for_each(Summarize::summarize);
                println!("last updated at: {}", Utc::now());
                thread::sleep(Duration::from_secs(delay));
            }
        }
        Commands::Status {group} => {
            let paths = db.get_repositories(group).unwrap();
            let repos: Vec<git2::Repository> = paths
                .into_iter()
                .map(git2::Repository::open)
                .collect::<Result<_, git2::Error>>()
                .unwrap();

            repos.iter().for_each(Summarize::summarize);
        }
    }
}
