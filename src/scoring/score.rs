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
    pub fn to_string(&self, use_score: bool) -> String {
        match self {
            Self::Ignored => "-".to_string(),
            Self::Scored { score, grade } => {
                if use_score {
                    format!("{}", score)
                } else {
                    format!("{:?}", grade)
                }
            }
        }
    }
}
