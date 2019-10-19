#[macro_use]
extern crate lazy_static;

mod commit;
mod config;
mod filter;
mod git;
mod platform;
mod scoring;

use commit::CommitInfo;
use config::read_config;
use git::GitRepository;
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

fn main() {
    platform_init();

    let config = read_config();
    let scorer = init_scorer();

    let repo = GitRepository::open(".");

    let printer = Printer {
        show_score: config.show_score(),
    };

    printer.print_header();

    let filters = config.filters();
    let max_commits = config.max_commits().unwrap_or(std::usize::MAX);

    repo.traverse(config.start_commit())
        .filter(|item| filters.accept(item.metadata()))
        .take(max_commits)
        .map(|item| item.parse())
        .for_each(|info| {
            let score = scorer.score(&info);
            printer.print_commit(&info, &score);
        });
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
