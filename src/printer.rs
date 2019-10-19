use crate::commit::CommitInfo;
use crate::scoring::score::{CommitScore, ScoreGrade};

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

    pub fn print_commit(&self, commit_info: &CommitInfo, score: &CommitScore) {
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

    fn colorize_score(&self, score: &CommitScore) -> ColoredString {
        let score_text = score.to_string(self.show_score);

        let score_color = match score {
            CommitScore::Ignored => Color::White,
            CommitScore::Scored { score: _, grade } => match grade {
                ScoreGrade::A => Color::BrightGreen,
                ScoreGrade::B => Color::BrightWhite,
                ScoreGrade::C => Color::BrightYellow,
                ScoreGrade::D => Color::BrightRed,
                ScoreGrade::F => Color::Red,
            },
        };

        score_text.color(score_color)
    }
}
