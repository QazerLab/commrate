/// A commit metadata, which is easy to obtain from
/// the repository without any heavy processing.
pub struct CommitMetadata {
    id: String,
    author: String,
    parents: usize,
}

impl CommitMetadata {
    pub fn new(id: String, author: String, parents: usize) -> CommitMetadata {
        CommitMetadata {
            id,
            author,
            parents,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn author(&self) -> &str {
        &self.author
    }

    pub fn parents(&self) -> usize {
        self.parents
    }
}
