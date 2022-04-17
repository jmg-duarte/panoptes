use colored::Colorize;

pub trait RepositoryExt {
    fn last_commit(&self) -> git2::Commit;
    fn file_statuses(&self) -> git2::Statuses;
}

impl RepositoryExt for git2::Repository {
    fn last_commit(&self) -> git2::Commit {
        // TODO: properly handle all errors
        self.head().unwrap().peel_to_commit().unwrap()
    }

    fn file_statuses(&self) -> git2::Statuses {
        let mut status_opts = git2::StatusOptions::new();
        status_opts
            .include_untracked(true)
            .show(git2::StatusShow::IndexAndWorkdir);

        // TODO: properly handle the Result
        self.statuses(Some(&mut status_opts)).unwrap()
    }
}

pub trait Summarize: RepositoryExt {
    fn summarize(&self, commit_opts: &CommitOptions);
}

impl Summarize for git2::Repository {
    fn summarize(&self, commit_opts: &CommitOptions) {
        println!(
            "{} [{}]",
            self.path().parent().unwrap().display().to_string().green(),
            self.head().unwrap().shorthand().unwrap().blue().bold()
        );

        let statuses = self.file_statuses();
        if !statuses.is_empty() {
            if !commit_opts.is_empty() {
                println!("File status:");
            }
            for s in statuses.iter() {
                if let Some(path) = s.path() {
                    println!("  {:#?} {}", s.status(), path.yellow());
                }
            }
        }

        if !commit_opts.is_empty() {
            let commit = self.last_commit();
            println!("Last commit:");
            if commit_opts.contains(CommitOptions::MESSAGE) {
                println!("  Message:");
                println!(
                    "    {}",
                    commit
                        .message()
                        .unwrap()
                        .split("\n")
                        .take(1)
                        .collect::<String>()
                );
            }
            if commit_opts.contains(CommitOptions::DATE) {
                println!("  Date:");
                let time = commit.time().seconds();
                println!("    {}", chrono::NaiveDateTime::from_timestamp(time, 0));
            }
            if commit_opts.contains(CommitOptions::HASH) {
                println!("  Hash:");
                println!("    {}", commit.id());
            }
        }
    }
}

bitflags::bitflags! {
    pub struct CommitOptions: usize {
        const DATE = 0b00000001;
        const HASH = 0b00000010;
        const MESSAGE = 0b00000100;
    }
}

impl CommitOptions {
    /// Return the bitflag length.
    pub const fn length() -> usize {
        let mut all = Self::all().bits();
        let mut idx = 0;
        while all != 0 {
            all >>= 1;
            idx += 1;
        }
        idx
    }
}

impl std::str::FromStr for CommitOptions {
    /// During parsing, unknown strings are ignored.
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut opts = Self::empty();

        for elem in s.to_ascii_lowercase().split(",").map(&str::trim) {
            match elem {
                "date" => opts |= CommitOptions::DATE,
                "hash" => opts |= CommitOptions::HASH,
                "message" => opts |= CommitOptions::MESSAGE,
                "all" => opts |= CommitOptions::all(),
                _ => continue,
            }
        }

        Ok(opts)
    }
}
