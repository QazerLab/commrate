use crate::{
    commit::CommitMetadata,
    scoring::{grade::GradeSpec, score::Score, scorer::ScoredCommit},
};

/// A chain of filters which can be applied to some commit at some stage
/// of evaluation. A type parameter D is specific for each stage (see the doc
/// for Filter::Descriptor associated type), so filters for different stages
/// cannot be grouped into single FilterChan.
pub struct FilterChain<D>(Vec<Box<dyn Filter<Descriptor = D>>>);

impl<D> FilterChain<D> {
    // TODO: consider using the associated type definition
    //
    // type Item = Box<dyn Filter<Descriptor = D>>;
    //
    // when Rust will support this.
    //
    // Tracking issue: https://github.com/rust-lang/rust/issues/8995

    pub fn new(filters: Vec<Box<dyn Filter<Descriptor = D>>>) -> Self {
        Self(filters)
    }

    pub fn accept(&self, descriptor: &D) -> bool {
        for filter in &self.0 {
            if !filter.accept(descriptor) {
                return false;
            }
        }

        true
    }
}

/// A single commit filter.
pub trait Filter {
    /// Filters may be applied at different stages of the
    /// commit evaluation pipeline. A descriptor is an object
    /// which contains the information required by filters
    /// at the specific stage.
    ///
    /// For example, late filters may check commit score, which
    /// is known only at the very end of the pipeline, while
    /// early filters require only some metadata like commit author.
    type Descriptor;

    fn accept(&self, descriptor: &Self::Descriptor) -> bool;
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

impl Filter for AuthorPreFilter {
    type Descriptor = CommitMetadata;

    fn accept(&self, metadata: &CommitMetadata) -> bool {
        self.author == metadata.author()
    }
}

/// A filter which accepts only non-merge commits.
pub struct MergePreFilter;

impl Filter for MergePreFilter {
    type Descriptor = CommitMetadata;

    fn accept(&self, metadata: &CommitMetadata) -> bool {
        metadata.parents() <= 1
    }
}

/// A post-filter for discarding commits based on their score.
pub struct GradePostFilter {
    spec: GradeSpec,
}

impl Filter for GradePostFilter {
    type Descriptor = ScoredCommit;

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
