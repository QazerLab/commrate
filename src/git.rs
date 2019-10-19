use crate::commit::{CommitInfo, CommitMetadata, DiffInfo, MessageInfo};

use colored::Colorize;
use git2::{Commit, DiffStats, Error, Repository, Revwalk};
use std::process::exit;

pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    pub fn open(location: &str) -> GitRepository {
        GitRepository {
            repo: git_expect(Repository::discover(location)),
        }
    }

    pub fn traverse(&self, start_commit: &str) -> GitTraversal {
        let mut revwalk = git_expect(self.repo.revwalk());
        let rev = git_expect(self.repo.revparse_single(start_commit));
        git_expect(revwalk.push(rev.id()));

        GitTraversal {
            repo: &self.repo,
            revwalk,
        }
    }
}

pub struct GitTraversal<'repo> {
    repo: &'repo Repository,
    revwalk: Revwalk<'repo>,
}

impl<'repo> Iterator for GitTraversal<'repo> {
    type Item = GitRepositoryItem<'repo>;

    fn next(&mut self) -> Option<GitRepositoryItem<'repo>> {
        self.revwalk.next().map(|commit_id| {
            let id = git_expect(commit_id);
            let commit = git_expect(self.repo.find_commit(id));

            let metadata = CommitMetadata::new(
                commit.id().to_string(),
                commit.author().name().unwrap().to_string(),
                commit.parent_count(),
            );

            GitRepositoryItem {
                repo: self.repo,
                metadata,
                commit,
            }
        })
    }
}

pub struct GitRepositoryItem<'repo> {
    repo: &'repo Repository,
    metadata: CommitMetadata,
    commit: Commit<'repo>,
}

impl<'repo> GitRepositoryItem<'repo> {
    pub fn metadata(&self) -> &CommitMetadata {
        &self.metadata
    }

    pub fn parse(self) -> CommitInfo {
        let msg_info = self
            .commit
            .message()
            .map(MessageInfo::new)
            .unwrap_or_default();

        if self.metadata.parents() >= 2 {
            return CommitInfo::new_from_merge(self.metadata, msg_info);
        }

        let parent = self.commit.parents().next();

        let tree = git_expect(self.commit.tree());
        let parent_tree = git_expect(parent.as_ref().map(|p| p.tree()).transpose());

        let diff = git_expect(
            self.repo
                .diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None),
        );

        let diff_stats = git_expect(diff.stats());
        let diff_info = parse_diff_stats(&diff_stats);

        CommitInfo::new(self.metadata, diff_info, msg_info)
    }
}

fn git_expect<T>(wrapped: Result<T, Error>) -> T {
    match wrapped {
        Ok(value) => value,
        Err(err) => {
            eprintln!("{}: {}", "error".red(), err.message());
            exit(1);
        }
    }
}

fn parse_diff_stats(stats: &DiffStats) -> DiffInfo {
    let insertions = stats.insertions();
    let deletions = stats.deletions();

    DiffInfo::new(insertions, deletions, insertions + deletions)
}
