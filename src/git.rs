use crate::commit::{CommitInfo, CommitMetadata, DiffInfo, MessageInfo};

use git2::{Commit, DiffStats, Error, Repository};

pub fn parse_commit<'repo>(
    commit: &'repo Commit<'repo>,
    repo: &'repo Repository,
) -> Result<CommitInfo<'repo>, Error> {
    let msg_info = commit.message().map(MessageInfo::new).unwrap_or_default();

    let metadata = CommitMetadata::new(
        commit.id().to_string(),
        commit.author().name().unwrap().to_string(),
        commit.parent_count(),
    );

    if metadata.parents() >= 2 {
        return Ok(CommitInfo::new_from_merge(metadata, msg_info));
    }

    let parent = commit.parents().next();

    let tree = commit.tree()?;
    let parent_tree = parent.as_ref().map(|p| p.tree()).transpose()?;

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
    let diff_stats = diff.stats()?;
    let diff_info = parse_diff_stats(&diff_stats);

    Ok(CommitInfo::new(metadata, diff_info, msg_info))
}

fn parse_diff_stats(stats: &DiffStats) -> DiffInfo {
    let insertions = stats.insertions();
    let deletions = stats.deletions();

    DiffInfo::new(insertions, deletions, insertions + deletions)
}
