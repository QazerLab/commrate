use crate::commit::{Commit, DiffInfo, MessageInfo, Metadata};

use colored::Colorize;
use git2::{Commit as GitCommit, DiffStats, Error, Repository, Revwalk};
use std::process::exit;

pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    pub fn open(location: &str) -> Self {
        Self {
            repo: git_expect(Repository::discover(location)),
        }
    }

    pub fn traverse(&self, start_commit: &str) -> GitTraversal<'_> {
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

            let metadata = Metadata::new(
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
    metadata: Metadata,
    commit: GitCommit<'repo>,
}

impl GitRepositoryItem<'_> {
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn parse(self) -> Commit {
        let msg_info = self
            .commit
            .message()
            .map(MessageInfo::new)
            .unwrap_or_default();

        if self.metadata.parents() >= 2 {
            return Commit::new_from_merge(self.metadata, msg_info);
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

        Commit::new(self.metadata, diff_info, msg_info)
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

    DiffInfo::new(insertions, deletions)
}
