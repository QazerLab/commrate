use crate::scoring::grade::Grade;

#[derive(Clone, Copy, Debug)]
pub enum Score {
    Ignored,

    Scored {
        score: u8,
        grade: Grade,
    },
}

impl Score {
    pub fn to_string(self, use_score: bool) -> String {
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
    fn ignored_score_is_rendered_as_dash() {
        let score = Score::Ignored;

        assert_eq!(score.to_string(true), "-");
        assert_eq!(score.to_string(false), "-");
    }

    #[test]
    fn score_is_rendered_as_grade() {
        let score = Score::Scored {
            score: 42,
            grade: Grade::C,
        };

        assert_eq!(score.to_string(false), "C");
    }

    #[test]
    fn score_is_rendered_as_number() {
        let score = Score::Scored {
            score: 42,
            grade: Grade::C,
        };

        assert_eq!(score.to_string(true), "42");
    }
}
