use std::{
    collections::HashMap,
    env::current_dir,
    io::{stdout, Write},
    path::PathBuf,
    str::FromStr,
    sync::mpsc::channel,
    time::Duration,
};

use clap::{Parser, Subcommand};

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use panoptes::{db::Database, stdout::StdoutExt};

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
    },
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
        Commands::Watch { group } => {
            let (snd, rcv) = channel();
            let mut watcher = watcher(snd, Duration::from_secs(3)).unwrap();

            let paths = db.get_repositories(group).unwrap();
            let repos = paths
                .into_iter()
                .map(|path| {
                    let repo = git2::Repository::open(path).unwrap();
                    watcher.watch(repo.path(), RecursiveMode::Recursive).unwrap();
                    repo
                })
                .collect();

            loop {
                stdout().clear_screen().unwrap();
                let event = rcv.recv().unwrap();
                match event {
                    DebouncedEvent::NoticeWrite(p) => {
                        fun_name(&repos, p);
                    }
                    DebouncedEvent::NoticeRemove(p) => {
                        fun_name(&repos, p);
                    }
                    DebouncedEvent::Create(p) => {
                        fun_name(&repos, p);
                    }
                    DebouncedEvent::Write(p) => {
                        fun_name(&repos, p);
                    }
                    DebouncedEvent::Chmod(p) => {
                        fun_name(&repos, p);
                    }
                    DebouncedEvent::Remove(p) => {
                        fun_name(&repos, p);
                    }
                    DebouncedEvent::Rename(_, _) => todo!(),
                    DebouncedEvent::Rescan => todo!(),
                    DebouncedEvent::Error(_, _) => todo!(),
                }
            }
        }
    }
}

fn fun_name(path_to_repo: &Vec<git2::Repository>, p: PathBuf) {
    for repo in path_to_repo.iter() {
        if p.starts_with(repo.path()) {
            let mut status_opts = git2::StatusOptions::new();
            status_opts
                .include_untracked(true)
                .show(git2::StatusShow::IndexAndWorkdir);

            println!("{} {}", repo.path().parent().unwrap().display(), repo.head().unwrap().shorthand().unwrap());

            let statuses = repo.statuses(Some(&mut status_opts)).unwrap();
            for s in statuses.iter() {
                if let Some(path) = s.path() {
                    println!(
                        "\t{:#016b} {:#?} {:#?}",
                        s.status().bits(),
                        s.status(),
                        path
                    );
                }
            }
        }
    }
}
