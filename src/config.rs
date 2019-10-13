use clap::{App, Arg, ArgMatches};
use git2::Commit;

pub trait CommitFilter {
    fn accept(&self, commit: &Commit) -> bool;
}

pub struct AuthorCommitFilter {
    author: String,
}

pub struct MergeCommitFilter;

pub struct AppConfig {
    commit_filters: Vec<Box<dyn CommitFilter>>,
    start_commit: String,
    max_commits: Option<u32>,
    show_score: bool,
}

impl AuthorCommitFilter {
    pub fn new(author: &str) -> AuthorCommitFilter {
        AuthorCommitFilter {
            author: author.to_owned(),
        }
    }
}

impl CommitFilter for AuthorCommitFilter {
    fn accept(&self, commit: &Commit) -> bool {
        match commit.author().name() {
            Some(name) => self.author == name,
            None => true,
        }
    }
}

impl CommitFilter for MergeCommitFilter {
    fn accept(&self, commit: &Commit) -> bool {
        commit.parent_count() <= 1
    }
}

impl AppConfig {
    pub fn filters(&self) -> &Vec<Box<dyn CommitFilter>> {
        &self.commit_filters
    }

    pub fn max_commits(&self) -> Option<u32> {
        self.max_commits
    }

    pub fn show_score(&self) -> bool {
        self.show_score
    }

    pub fn start_commit(&self) -> &str {
        &self.start_commit
    }
}

pub fn read_config() -> AppConfig {
    let app = init_clap_app();
    let matches = app.get_matches();
    let commit_filters = create_filters(&matches);
    let max_commits = read_commits_number(&matches);
    let start_commit = matches.value_of("commit").unwrap_or("HEAD").to_string();
    let show_score = matches.occurrences_of("score") > 0;

    AppConfig {
        commit_filters,
        start_commit,
        max_commits,
        show_score,
    }
}

fn init_clap_app() -> App<'static, 'static> {
    App::new("commrate")
        .version(env!("CARGO_PKG_VERSION"))
        .about("The tool for scoring and rating Git commits.")
        .arg(
            Arg::with_name("commit")
                .value_name("START_COMMIT")
                .default_value("HEAD")
                .help("Commit ID or reference to start from"),
        )
        .arg(
            Arg::with_name("author")
                .short("a")
                .long("author")
                .value_name("AUTHOR")
                .help("Filters by commit author"),
        )
        .arg(
            Arg::with_name("merges")
                .short("m")
                .long("merges")
                .help("Includes (but not scores) merge commits into the output"),
        )
        .arg(
            Arg::with_name("number")
                .short("n")
                .long("number")
                .value_name("NUMBER")
                .validator(|arg| {
                    if let Ok(_) = arg.parse::<u32>() {
                        return Ok(());
                    }

                    Err("must be a non-negative number".to_string())
                })
                .help("Maximum number of commits to show"),
        )
        .arg(
            Arg::with_name("score")
                .short("s")
                .long("score")
                .help("Shows numeric scores instead of discrete grades"),
        )
}

fn create_filters(matches: &ArgMatches) -> Vec<Box<dyn CommitFilter>> {
    let mut commit_filters: Vec<Box<dyn CommitFilter>> = Vec::new();
    if let Some(author) = matches.value_of("author") {
        let filter = AuthorCommitFilter::new(author);
        commit_filters.push(Box::new(filter));
    }

    if matches.occurrences_of("merges") == 0 {
        let filter = MergeCommitFilter;
        commit_filters.push(Box::new(filter));
    }

    commit_filters
}

fn read_commits_number(matches: &ArgMatches) -> Option<u32> {
    match matches.value_of("number") {
        Some(arg) => Some(arg.parse().unwrap()),
        None => None,
    }
}
