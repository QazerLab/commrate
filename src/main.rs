#[macro_use]
extern crate lazy_static;

mod commit;
mod config;
mod filter;
mod git;
mod platform;
mod printer;
mod scoring;

use config::read_config;
use git::GitRepository;
use platform::platform_init;
use printer::Printer;
use scoring::{
    rule::{
        BodyLenRule, BodyPresenceRule, BodyWrappingRule, MetadataLinesRule, SubjectBodyBreakRule,
        SubjectRule,
    },
    scorer::{Scorer, ScorerBuilder},
};

fn main() {
    platform_init();

    let config = read_config();
    let scorer = init_scorer();

    let repo = GitRepository::open(".");
    let printer = Printer::new(config.show_score());

    printer.print_header();

    let pre_filters = config.pre_filters();
    let max_commits = config.max_commits().unwrap_or(std::usize::MAX);

    repo.traverse(config.start_commit())
        .filter(|item| pre_filters.accept(item.metadata()))
        .take(max_commits)
        .map(|item| item.parse())
        .for_each(|info| {
            let score = scorer.score(&info);
            printer.print_commit(&info, &score);
        });
}

fn init_scorer() -> Scorer {
    ScorerBuilder::new()
        .with_rule(SubjectRule, 0.3)
        .with_rule(BodyPresenceRule, 0.1)
        .with_rule(SubjectBodyBreakRule, 0.1)
        .with_rule(BodyLenRule, 0.25)
        .with_rule(BodyWrappingRule, 0.25)
        .with_rule(MetadataLinesRule, 0.05)
        .build()
}
