use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Grade {
    F,
    D,
    C,
    B,
    A,
}

/// A spec for matching grade.
#[derive(Debug, PartialEq)]
pub struct GradeSpec {
    grade: Grade,
    rel: Relation,
}

impl FromStr for GradeSpec {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();

        let grade = match chars.next() {
            Some(chr) => match chr {
                'A' | 'a' => Grade::A,
                'B' | 'b' => Grade::B,
                'C' | 'c' => Grade::C,
                'D' | 'd' => Grade::D,
                'F' | 'f' => Grade::F,
                _ => return Err("grade must be one of: A, B, C, D, F"),
            },

            None => return Err("grade must be specified"),
        };

        let rel = match chars.next() {
            Some(chr) => match chr {
                '+' => Relation::Ge,
                '-' => Relation::Le,
                _ => return Err("grade relation must be one of: +, -, <empty>"),
            },

            None => Relation::Eq,
        };

        if let Some(_) = chars.next() {
            return Err("grade specification should not contain extra characters");
        }

        Ok(GradeSpec { grade, rel })
    }
}

impl GradeSpec {
    pub fn matches(&self, grade: Grade) -> bool {
        match self.rel {
            Relation::Eq => grade == self.grade,
            Relation::Ge => grade >= self.grade,
            Relation::Le => grade <= self.grade,
        }
    }
}

/// A relation specification between different scores/grades.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Relation {
    Eq,
    Le,
    Ge,
}

#[cfg(test)]
mod tests {
    use super::*;
    use Grade::*;
    use Relation::*;

    #[test]
    fn grades_are_ordered_from_f_to_a() {
        assert!(D > F);
        assert!(C > D);
        assert!(B > C);
        assert!(A > B);

        // The rest is guaranteed by PartialOrd's transitivity.
    }

    #[test]
    fn invalid_grade_spec_returns_error() {
        assert!(GradeSpec::from_str("").is_err());
        assert!(GradeSpec::from_str("+").is_err());
        assert!(GradeSpec::from_str("-").is_err());
        assert!(GradeSpec::from_str("C++").is_err());
        assert!(GradeSpec::from_str("Abyrvalg!").is_err());
    }

    #[test]
    fn valid_grade_spec_is_parsed_successfully() {
        for &grade in [A, B, C, D, F].iter() {
            for &rel in [Eq, Ge, Le].iter() {
                let rel_str = match rel {
                    Eq => "",
                    Le => "-",
                    Ge => "+",
                };

                // Render current grade and relation combination into
                // both possible formats, then try to parse it into GradeSpec.
                let input = format!("{:?}{}", grade, rel_str);
                let input_lower = input.to_ascii_lowercase();

                let expected = GradeSpec { grade, rel };

                assert_eq!(GradeSpec::from_str(&input).unwrap(), expected);
                assert_eq!(GradeSpec::from_str(&input_lower).unwrap(), expected);
            }
        }
    }

    #[test]
    fn grade_spec_matches_eq() {
        let spec = GradeSpec { grade: C, rel: Eq };

        assert!(!spec.matches(A));
        assert!(!spec.matches(B));
        assert!(spec.matches(C));
        assert!(!spec.matches(D));
        assert!(!spec.matches(F));
    }

    #[test]
    fn grade_spec_matches_ge() {
        let spec = GradeSpec { grade: C, rel: Ge };

        assert!(spec.matches(A));
        assert!(spec.matches(B));
        assert!(spec.matches(C));
        assert!(!spec.matches(D));
        assert!(!spec.matches(F));
    }

    #[test]
    fn grade_spec_matches_le() {
        let spec = GradeSpec { grade: C, rel: Le };

        assert!(!spec.matches(A));
        assert!(!spec.matches(B));
        assert!(spec.matches(C));
        assert!(spec.matches(D));
        assert!(spec.matches(F));
    }
}
