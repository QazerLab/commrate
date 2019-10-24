use crate::{
    commit::CommitMetadata,
    scoring::{
        score::{GradeSpec, Score},
        scorer::ScoredCommit,
    },
};

/// A chain of commit filters for discarding unneeded commits
/// as early as it is possible.
pub struct PreFilters(Vec<Box<dyn PreFilter>>);

impl PreFilters {
    pub fn new(filters: Vec<Box<dyn PreFilter>>) -> PreFilters {
        PreFilters(filters)
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
    pub fn new(author: &str) -> AuthorPreFilter {
        AuthorPreFilter {
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
    pub fn new(filters: Vec<Box<dyn PostFilter>>) -> PostFilters {
        PostFilters(filters)
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

pub struct GradePostFilter {
    spec: GradeSpec,
}

impl PostFilter for GradePostFilter {
    fn accept(&self, commit: &ScoredCommit) -> bool {
        match commit.score() {
            Score::Ignored => true,
            Score::Scored { score: _, grade } => self.spec.matches(*grade),
        }
    }
}

impl GradePostFilter {
    pub fn new(spec: GradeSpec) -> GradePostFilter {
        GradePostFilter { spec }
    }
}
