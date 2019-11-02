/// Statistics of specific diff.
pub struct DiffInfo {
    insertions: usize,
    deletions: usize,
    diff_total: usize,
}

impl DiffInfo {
    pub fn new(insertions: usize, deletions: usize) -> DiffInfo {
        DiffInfo {
            insertions,
            deletions,
            diff_total: insertions + deletions,
        }
    }

    pub fn insertions(&self) -> usize {
        self.insertions
    }
    pub fn deletions(&self) -> usize {
        self.deletions
    }
    pub fn diff_total(&self) -> usize {
        self.diff_total
    }
}
