use crate::scoring::{Grade, Score, ScoredCommit};

use colored::{Color, ColoredString, Colorize};

pub struct Printer {
    show_score: bool,
}

impl Printer {
    pub fn new(show_score: bool) -> Self {
        Self { show_score }
    }

    pub fn print_header(&self) {
        let score_title = if self.show_score { "SCORE" } else { "GRADE" };

        println!("{:12} {:5} {:19} SUBJECT", "COMMIT", score_title, "AUTHOR");
    }

    pub fn print_commit(&self, scored_commit: &ScoredCommit) {
        let commit = scored_commit.commit();
        let score = scored_commit.score();
        let metadata = commit.metadata();
        let msg_info = commit.msg_info();
        let score_colored = self.colorize_score(score);

        println!(
            "{:.12} {:<5} {:19.19} {}",
            metadata.id().yellow(),
            score_colored,
            metadata.author(),
            msg_info.subject().unwrap_or("")
        );
    }

    fn colorize_score(&self, score: Score) -> ColoredString {
        let score_text = score.to_string(self.show_score);

        let score_color = match score {
            Score::Ignored => Color::White,
            Score::Scored { grade, .. } => match grade {
                Grade::A => Color::BrightGreen,
                Grade::B => Color::BrightWhite,
                Grade::C => Color::BrightYellow,
                Grade::D => Color::BrightRed,
                Grade::F => Color::Red,
            },
        };

        score_text.color(score_color)
    }
}
