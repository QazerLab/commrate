use crate::commit::{CommitClass, CommitInfo};
use crate::scoring::{
    rule::ScoringRule,
    score::{CommitScore, ScoreGrade},
};

pub struct Scorer {
    rules: Vec<ScorerItem>,
}

pub struct ScorerBuilder {
    rules: Vec<ScorerItem>,
}

struct ScorerItem {
    rule: Box<dyn ScoringRule>,
    weight: f32,
}

impl ScorerBuilder {
    pub fn new() -> ScorerBuilder {
        ScorerBuilder { rules: Vec::new() }
    }

    pub fn with_rule(mut self, rule: Box<dyn ScoringRule>, weight: f32) -> ScorerBuilder {
        self.rules.push(ScorerItem { rule, weight });
        self
    }

    pub fn build(self) -> Scorer {
        Scorer { rules: self.rules }
    }
}

impl Scorer {
    pub fn score(&self, commit: &CommitInfo) -> CommitScore {
        if commit.classes().as_set().contains(CommitClass::MergeCommit) {
            return CommitScore::Ignored;
        }

        let mut score_accum = 0.0;

        for item in &self.rules {
            score_accum += 100.0 * item.rule.score(commit) * item.weight;
        }

        let score = if score_accum > 100.0 {
            100
        } else {
            score_accum.round() as u8
        };

        let grade = match score {
            0..=19 => ScoreGrade::F,
            20..=39 => ScoreGrade::D,
            40..=59 => ScoreGrade::C,
            60..=79 => ScoreGrade::B,
            _ => ScoreGrade::A,
        };

        CommitScore::Scored { score, grade }
    }
}
