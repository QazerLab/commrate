#[macro_use]
extern crate lazy_static;

mod commit;
mod config;
mod platform;
mod scoring;

use commit::{parse_commit, CommitInfo};
use config::read_config;
use platform::platform_init;
use scoring::{
    rule::{
        BodyLenRule, BodyPresenceRule, BodyWrappingRule, MetadataLinesRule, SubjectBodyBreakRule,
        SubjectRule,
    },
    score::CommitScore,
    scorer::{Scorer, ScorerBuilder},
};

use colored::Colorize;
use git2::{Error, Repository, Revwalk};
use std::process::exit;

fn main() {
    platform_init();

    let config = read_config();
    let filters = config.filters();

    let scorer = init_scorer();

    // TODO: push Git stuff down behind the abstraction level.
    let repo = load_repo(".");
    let revwalk = init_revwalk(&repo, config.start_commit());

    let printer = Printer {
        show_score: config.show_score(),
    };

    printer.print_header();

    let max_commits = config.max_commits().unwrap_or(std::u32::MAX);
    let mut commit_num = 0;

    'commits: for commit_id in revwalk {
        if commit_num == max_commits {
            break;
        }

        let id = git_expect(commit_id);
        let commit = git_expect(repo.find_commit(id));

        for filter in filters {
            if !filter.accept(&commit) {
                continue 'commits;
            }
        }

        let commit_info = git_expect(parse_commit(&commit, &repo));
        let score = scorer.score(&commit_info);
        printer.print_commit(&commit_info, &score);

        commit_num += 1;
    }
}

fn init_scorer() -> Scorer {
    ScorerBuilder::new()
        .with_rule(Box::new(SubjectRule), 0.3)
        .with_rule(Box::new(BodyPresenceRule), 0.1)
        .with_rule(Box::new(SubjectBodyBreakRule), 0.1)
        .with_rule(Box::new(BodyLenRule), 0.25)
        .with_rule(Box::new(BodyWrappingRule), 0.25)
        .with_rule(Box::new(MetadataLinesRule), 0.05)
        .build()
}

fn load_repo(location: &str) -> Repository {
    git_expect(Repository::discover(location))
}

fn init_revwalk<'repo>(repo: &'repo Repository, start: &str) -> Revwalk<'repo> {
    let mut revwalk = git_expect(repo.revwalk());
    let rev = git_expect(repo.revparse_single(start));
    git_expect(revwalk.push(rev.id()));

    revwalk
}

fn git_expect<T>(wrapped: Result<T, Error>) -> T {
    match wrapped {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{}: {}", "error".red(), err.message());
            exit(1);
        }
    }
}

struct Printer {
    show_score: bool,
}

impl Printer {
    pub fn print_header(&self) {
        let score_title = if self.show_score { "SCORE" } else { "GRADE" };

        println!(
            "{:7} {:5} {:19} {}",
            "COMMIT", score_title, "AUTHOR", "SUBJECT"
        );
    }

    fn print_commit(&self, commit_info: &CommitInfo, score: &CommitScore) {
        let metadata = commit_info.metadata();
        let msg_info = commit_info.msg_info();

        println!(
            "{:7.7} {:<5} {:19.19} {}",
            metadata.id().yellow(),
            score.to_string(self.show_score),
            metadata.author(),
            msg_info.subject().unwrap_or("")
        );
    }
}
