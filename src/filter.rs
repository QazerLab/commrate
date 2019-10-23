use crate::commit::CommitMetadata;

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
