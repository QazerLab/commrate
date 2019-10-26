use crate::{
    commit::CommitMetadata,
    scoring::{grade::GradeSpec, score::Score, scorer::ScoredCommit},
};

/// A chain of commit filters for discarding unneeded commits
/// as early as it is possible.
pub struct PreFilters(Vec<Box<dyn PreFilter>>);

impl PreFilters {
    pub fn new(filters: Vec<Box<dyn PreFilter>>) -> Self {
        Self(filters)
    }

    pub fn accept(&self, metadata: &CommitMetadata) -> bool {
        for filter in &self.0 {
            if !filter.accept(metadata) {
                return false;
            }
        }

        true
    }
}

/// A single commit filter.
pub trait PreFilter {
    fn accept(&self, metadata: &CommitMetadata) -> bool;
}

/// A filter which accepts only commits with specific author.
pub struct AuthorPreFilter {
    author: String,
}

impl AuthorPreFilter {
    pub fn new(author: &str) -> Self {
        Self {
            author: author.to_owned(),
        }
    }
}

impl PreFilter for AuthorPreFilter {
    fn accept(&self, metadata: &CommitMetadata) -> bool {
        self.author == metadata.author()
    }
}

/// A filter which accepts only non-merge commits.
pub struct MergePreFilter;

impl PreFilter for MergePreFilter {
    fn accept(&self, metadata: &CommitMetadata) -> bool {
        metadata.parents() <= 1
    }
}

/// A chain of commit filters for discarding unneeded commits
/// after they are parsed and scored.
pub struct PostFilters(Vec<Box<dyn PostFilter>>);

impl PostFilters {
    pub fn new(filters: Vec<Box<dyn PostFilter>>) -> Self {
        Self(filters)
    }

    pub fn accept(&self, commit: &ScoredCommit) -> bool {
        for filter in &self.0 {
            if !filter.accept(commit) {
                return false;
            }
        }

        true
    }
}

/// A single commit post-filter.
pub trait PostFilter {
    fn accept(&self, commit: &ScoredCommit) -> bool;
}

/// A post-filter for discarding commits based on their score.
pub struct GradePostFilter {
    spec: GradeSpec,
}

impl PostFilter for GradePostFilter {
    fn accept(&self, commit: &ScoredCommit) -> bool {
        match commit.score() {
            Score::Ignored => true,
            Score::Scored { grade, .. } => self.spec.matches(grade),
        }
    }
}

impl GradePostFilter {
    pub fn new(spec: GradeSpec) -> GradePostFilter {
        GradePostFilter { spec }
    }
}
