use crate::commit::CommitInfo;
use crate::scoring::score::{Score, Grade};

use colored::{Color, ColoredString, Colorize};

pub struct Printer {
    show_score: bool,
}

impl Printer {
    pub fn new(show_score: bool) -> Printer {
        Printer { show_score }
    }

    pub fn print_header(&self) {
        let score_title = if self.show_score { "SCORE" } else { "GRADE" };

        println!(
            "{:7} {:5} {:19} {}",
            "COMMIT", score_title, "AUTHOR", "SUBJECT"
        );
    }

    pub fn print_commit(&self, commit_info: &CommitInfo, score: &Score) {
        let metadata = commit_info.metadata();
        let msg_info = commit_info.msg_info();
        let score_colored = self.colorize_score(score);

        println!(
            "{:7.7} {:<5} {:19.19} {}",
            metadata.id().yellow(),
            score_colored,
            metadata.author(),
            msg_info.subject().unwrap_or("")
        );
    }

    fn colorize_score(&self, score: &Score) -> ColoredString {
        let score_text = score.to_string(self.show_score);

        let score_color = match score {
            Score::Ignored => Color::White,
            Score::Scored { score: _, grade } => match grade {
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
