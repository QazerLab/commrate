use crate::commit::{CommitClass, CommitInfo};
use crate::scoring::{
    grade::Grade,
    rule::Rule,
    score::Score,
};

pub struct Scorer {
    rules: Vec<ScorerItem>,
}

pub struct ScorerBuilder {
    rules: Vec<ScorerItem>,
}

struct ScorerItem {
    rule: Box<dyn Rule>,
    weight: f32,
}

impl ScorerBuilder {
    pub fn new() -> ScorerBuilder {
        ScorerBuilder { rules: Vec::new() }
    }

    pub fn with_rule<R>(mut self, rule: R, weight: f32) -> ScorerBuilder
    where
        R: Rule + 'static,
    {
        self.rules.push(ScorerItem {
            rule: Box::new(rule),
            weight,
        });

        self
    }

    pub fn build(self) -> Scorer {
        Scorer { rules: self.rules }
    }
}

impl Scorer {
    pub fn score(&self, commit: CommitInfo) -> ScoredCommit {
        let score = self.score_internal(&commit);

        ScoredCommit { commit, score }
    }

    fn score_internal(&self, commit: &CommitInfo) -> Score {
        if commit.classes().as_set().contains(CommitClass::MergeCommit) {
            return Score::Ignored;
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
            0..=19 => Grade::F,
            20..=39 => Grade::D,
            40..=59 => Grade::C,
            60..=79 => Grade::B,
            _ => Grade::A,
        };

        Score::Scored { score, grade }
    }
}

pub struct ScoredCommit {
    commit: CommitInfo,
    score: Score,
}

impl ScoredCommit {
    pub fn commit(&self) -> &CommitInfo {
        &self.commit
    }

    pub fn score(&self) -> Score {
        self.score
    }
}
