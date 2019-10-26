use crate::{
    filter::{
        AuthorPreFilter, GradePostFilter, MergePreFilter, PostFilter, PostFilters, PreFilter,
        PreFilters,
    },
    scoring::grade::GradeSpec,
};

use clap::{App, Arg, ArgMatches};
use std::str::FromStr;

pub struct AppConfig {
    pre_filters: PreFilters,
    post_filters: PostFilters,
    start_commit: String,
    max_commits: Option<usize>,
    show_score: bool,
}

impl AppConfig {
    pub fn pre_filters(&self) -> &PreFilters {
        &self.pre_filters
    }

    pub fn post_filters(&self) -> &PostFilters {
        &self.post_filters
    }

    pub fn max_commits(&self) -> Option<usize> {
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
    let pre_filters = create_pre_filters(&matches);
    let post_filters = create_post_filters(&matches);
    let max_commits = read_commits_number(&matches);
    let start_commit = matches.value_of("commit").unwrap_or("HEAD").to_string();
    let show_score = matches.occurrences_of("score") > 0;

    AppConfig {
        pre_filters,
        post_filters,
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
            Arg::with_name("grades")
                .short("g")
                .long("grades")
                .value_name("GRADE_SPEC")
                .validator(try_parse::<GradeSpec>)
                .help("Filters by commit grade"),
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
                .validator(try_parse::<usize>)
                .help("Maximum number of commits to show"),
        )
        .arg(
            Arg::with_name("score")
                .short("s")
                .long("score")
                .help("Shows numeric scores instead of discrete grades"),
        )
}

/// A generic parseability validator for Clap arguments.
///
/// It is required for the following reasons:
///
/// * no matter what type of successfull parsing result is,
///   Clap always expects just Ok(());
/// * in case of error Clap expects Err(String), but different
///   target types have different Err associate types in their
///   FromStr implementations, so we need to do the conversion
///   from T::Err to Err(String) in generic manner.
fn try_parse<T>(arg: String) -> Result<(), String>
where
    T: FromStr,
    T::Err: ToString,
{
    arg.parse::<T>().map_err(|s| s.to_string()).map(|_| ())
}

fn create_pre_filters(matches: &ArgMatches) -> PreFilters {
    let mut commit_filters: Vec<Box<dyn PreFilter>> = Vec::new();

    if let Some(author) = matches.value_of("author") {
        let filter = AuthorPreFilter::new(author);
        commit_filters.push(Box::new(filter));
    }

    if matches.occurrences_of("merges") == 0 {
        commit_filters.push(Box::new(MergePreFilter));
    }

    PreFilters::new(commit_filters)
}

fn create_post_filters(matches: &ArgMatches) -> PostFilters {
    let mut commit_filters: Vec<Box<dyn PostFilter>> = Vec::new();

    if let Some(grades) = matches.value_of("grades") {
        let spec = grades.parse::<GradeSpec>().unwrap();
        let filter = GradePostFilter::new(spec);
        commit_filters.push(Box::new(filter));
    }

    PostFilters::new(commit_filters)
}

fn read_commits_number(matches: &ArgMatches) -> Option<usize> {
    matches.value_of("number").map(|arg| arg.parse().unwrap())
}
