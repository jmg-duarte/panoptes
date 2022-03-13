use colored::Colorize;

pub trait Summarize {
    fn summarize(&self);
}

impl Summarize for git2::Repository {
    fn summarize(&self) {
        let mut status_opts = git2::StatusOptions::new();
        status_opts
            .include_untracked(true)
            .show(git2::StatusShow::IndexAndWorkdir);

        println!(
            "{} [{}]",
            self.path().parent().unwrap().display().to_string().green(),
            self.head().unwrap().shorthand().unwrap().blue().bold()
        );

        let statuses = self.statuses(Some(&mut status_opts)).unwrap();
        for s in statuses.iter() {
            if let Some(path) = s.path() {
                println!("  {:#?} {}", s.status(), path.yellow());
            }
        }
    }
}
