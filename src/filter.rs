use crate::commit::CommitMetadata;

/// A chain of commit filters for discarding unneeded commits
/// as early as it is possible.
pub struct CommitFilters(Vec<Box<dyn CommitFilter>>);

impl CommitFilters {
    pub fn new(filters: Vec<Box<dyn CommitFilter>>) -> CommitFilters {
        CommitFilters(filters)
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
pub trait CommitFilter {
    fn accept(&self, metadata: &CommitMetadata) -> bool;
}

/// A filter which accepts only commits with specific author.
pub struct AuthorCommitFilter {
    author: String,
}

impl AuthorCommitFilter {
    pub fn new(author: &str) -> AuthorCommitFilter {
        AuthorCommitFilter {
            author: author.to_owned(),
        }
    }
}

impl CommitFilter for AuthorCommitFilter {
    fn accept(&self, metadata: &CommitMetadata) -> bool {
        self.author == metadata.author()
    }
}

/// A filter which accepts only non-merge commits.
pub struct MergeCommitFilter;

impl CommitFilter for MergeCommitFilter {
    fn accept(&self, metadata: &CommitMetadata) -> bool {
        metadata.parents() <= 1
    }
}
