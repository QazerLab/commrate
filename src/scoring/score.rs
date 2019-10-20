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

#[derive(Debug, PartialEq, PartialOrd)]
pub enum ScoreGrade {
    F,
    D,
    C,
    B,
    A,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grades_are_ordered_from_f_to_a() {
        let f = ScoreGrade::F;
        let d = ScoreGrade::D;
        let c = ScoreGrade::C;
        let b = ScoreGrade::B;
        let a = ScoreGrade::A;

        // The rest is guaranteed by PartialOrd's transitivity.
        assert!(d > f);
        assert!(c > d);
        assert!(b > c);
        assert!(a > b);
    }

    #[test]
    fn ignored_score_is_rendered_as_dash() {
        let score = CommitScore::Ignored;

        assert!(score.to_string(true) == "-");
        assert!(score.to_string(false) == "-");
    }

    #[test]
    fn score_is_rendered_as_grade() {
        let score = CommitScore::Scored {
            score: 42,
            grade: ScoreGrade::C,
        };

        assert!(score.to_string(false) == "C");
    }

    #[test]
    fn score_is_rendered_as_number() {
        let score = CommitScore::Scored {
            score: 42,
            grade: ScoreGrade::C,
        };

        assert!(score.to_string(true) == "42");
    }
}
