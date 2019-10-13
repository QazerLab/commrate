use colored::{Color, ColoredString, Colorize};

#[derive(Debug)]
pub enum CommitScore {
    Ignored,

    // XXX: this attribute is a workaround for compiler bug:
    //
    // https://github.com/rust-lang/rust/issues/64362
    //
    // Remove this attribute when the bug will be fixed.
    #[allow(dead_code)]
    Scored {
        score: u8,
        grade: ScoreGrade,
    },
}

#[derive(Debug)]
pub enum ScoreGrade {
    A,
    B,
    C,
    D,
    F,
}

impl CommitScore {
    pub fn to_string(&self, use_score: bool) -> ColoredString {
        let score_color = match self {
            Self::Ignored => Color::White,
            Self::Scored { score: _, grade } => match grade {
                ScoreGrade::A => Color::BrightGreen,
                ScoreGrade::B => Color::BrightWhite,
                ScoreGrade::C => Color::BrightYellow,
                ScoreGrade::D => Color::BrightRed,
                ScoreGrade::F => Color::Red,
            },
        };

        let score_text = match self {
            Self::Ignored => "-".to_string(),
            Self::Scored { score, grade } => {
                if use_score {
                    format!("{}", score)
                } else {
                    format!("{:?}", grade)
                }
            }
        };

        score_text.color(score_color)
    }
}
