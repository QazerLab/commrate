use enumset::{EnumSet, EnumSetType};
use git2::{Commit, DiffStats, Error, Repository};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

/// Maximum diff size (lines total) for short commits.
pub const SHORT_COMMIT_LENGTH: usize = 25;

/// Commits of different nature require special treatment
/// disregarging the fact that their actual properties like
/// diff length or message length are the same: having some
/// special *semantics* makes these commits not like the
/// other ones.
///
/// Comments for each case of this enum explain, why specific
/// semantics of specific commit makes it special.
#[derive(EnumSetType, Debug)]
pub enum CommitClass {
    MergeCommit,

    /// Initial commits usually do not have anything else
    /// a subject "Initial commit" in the message, though
    /// they frequently have huge diff.
    InitialCommit,

    /// Short commits may contain some primitive change
    /// which does not require additional explanations:
    /// version bumps, typo fixes, etc.
    ///
    /// No penalty for message body should be applied to
    /// such commits.
    ShortCommit,

    /// Commits whose sole purpose is renaming some file
    /// or piece of code (e.g. function) usually do not require
    /// additional explanations and may be described with a
    /// single subject line, e.g.
    ///
    /// "Rename Foo::bar() to Foo::baz()"
    /// "Rename src/module to src/another_module"
    ///
    /// Such commits could be pretty long though, so they
    /// require special treatment.
    RenameCommit,
}

/// A newtype wrapper for implementing Display.
pub struct CommitClasses(EnumSet<CommitClass>);

pub struct CommitMetadata {
    id: String,
    author: String,
    parents: usize,
}

pub struct DiffInfo {
    insertions: usize,
    deletions: usize,
    diff_total: usize,
}

#[derive(Default, Debug)]
pub struct MessageInfo<'repo> {
    subject: Option<&'repo str>,
    break_after_subject: bool,
    body_len: usize,
    body_lines: usize,
    body_unwrapped_lines: usize,
    metadata_lines: usize,
}

pub struct CommitInfo<'repo> {
    metadata: CommitMetadata,
    diff_info: Option<DiffInfo>,
    msg_info: MessageInfo<'repo>,
    classes: CommitClasses,
}

lazy_static! {
    static ref META_KEYS: HashSet<&'static str> = {
        let mut keys = HashSet::new();

        keys.insert("acked-by");
        keys.insert("analyzed-by");
        keys.insert("approved-by");
        keys.insert("assisted-by");
        keys.insert("based-on");
        keys.insert("bisected-by");
        keys.insert("caught-by");
        keys.insert("cc");
        keys.insert("checked-by");
        keys.insert("co-developed-by");
        keys.insert("fixed-by");
        keys.insert("fixes");
        keys.insert("found-by");
        keys.insert("investigated-by");
        keys.insert("link");
        keys.insert("rebased-by");
        keys.insert("reported-by");
        keys.insert("reviewed-by");
        keys.insert("sent-by");
        keys.insert("signed-off-by");
        keys.insert("sponsored-by");
        keys.insert("submitted-by");
        keys.insert("suggested-by");
        keys.insert("tested-by");
        keys.insert("triaged-by");
        keys.insert("written-by");

        keys
    };
}

impl CommitClasses {
    pub fn as_set(&self) -> &EnumSet<CommitClass> {
        &self.0
    }
}

impl Display for CommitClasses {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let set_len = self.as_set().len();
        let mut buf = String::with_capacity(set_len);
        for class in self.0 {
            buf.push(match class {
                CommitClass::MergeCommit => 'M',
                CommitClass::InitialCommit => 'I',
                CommitClass::RenameCommit => 'R',
                CommitClass::ShortCommit => 'S',
            });
        }

        Display::fmt(&buf, f)
    }
}

impl CommitMetadata {
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

impl DiffInfo {
    fn new(stats: &DiffStats) -> DiffInfo {
        let insertions = stats.insertions();
        let deletions = stats.deletions();
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

impl<'repo> MessageInfo<'repo> {
    pub fn new(raw_message: &str) -> MessageInfo {
        let mut subject: Option<&str> = None;
        let mut break_after_subject = false;
        let mut body_len = 0;
        let mut body_lines = 0;
        let mut body_unwrapped_lines = 0;
        let mut metadata_lines = 0;

        // Here we rely on line numbers, as Git strips
        // leading and trailing empty lines during commit.
        // This means, that the subject is always line 0.
        let mut line_num = 0;
        for line in raw_message.lines() {
            if line_num == 0 {
                subject = Some(line);
                line_num += 1;
                continue;
            }

            if line_num == 1 {
                break_after_subject = line.is_empty();
            }

            if let Some(meta_key) = line.split(':').next() {
                let key_lower = meta_key.trim().to_ascii_lowercase();
                if META_KEYS.contains(key_lower.as_str()) {
                    metadata_lines += 1;
                    continue;
                }
            }

            let line_len = line.len();
            body_len += line_len;
            body_lines += 1;
            if line_len > 80 {
                body_unwrapped_lines += 1;
            }

            line_num += 1;
        }

        MessageInfo {
            subject,
            break_after_subject,
            body_len,
            body_lines,
            body_unwrapped_lines,
            metadata_lines,
        }
    }

    pub fn subject(&self) -> Option<&str> {
        self.subject
    }

    pub fn break_after_subject(&self) -> bool {
        self.break_after_subject
    }

    pub fn body_len(&self) -> usize {
        self.body_len
    }

    pub fn body_lines(&self) -> usize {
        self.body_lines
    }

    pub fn body_unwrapped_lines(&self) -> usize {
        self.body_unwrapped_lines
    }

    pub fn metadata_lines(&self) -> usize {
        self.metadata_lines
    }
}

impl<'repo> CommitInfo<'repo> {
    pub fn metadata(&self) -> &CommitMetadata {
        &self.metadata
    }

    pub fn diff_info(&self) -> &Option<DiffInfo> {
        &self.diff_info
    }

    pub fn msg_info(&self) -> &MessageInfo {
        &self.msg_info
    }

    pub fn classes(&self) -> &CommitClasses {
        &self.classes
    }
}

fn do_classify_commit(
    metadata: &CommitMetadata,
    diff_info: &DiffInfo,
    msg_info: &MessageInfo,
) -> EnumSet<CommitClass> {
    let mut classes = EnumSet::new();

    if metadata.parents() == 0 {
        classes.insert(CommitClass::InitialCommit);
    }

    if diff_info.diff_total() < SHORT_COMMIT_LENGTH {
        classes.insert(CommitClass::ShortCommit);
    }

    // XXX: detection of rename commits is a best-effort attempt
    // and may produce both false positives and false negatives.
    //
    // False negatives are usual for cases when the renaming is
    // accompanied with some additional change. In most (if not all)
    // such cases this additonal change should be in the separate
    // commit, so these false negatives still *do* deserve an attention.
    //
    // False positives are extremely rare, so let's pretend they
    // are absent. At the end of the day, no one will die due to
    // one commit of thousands being *overscored*.
    if diff_info.deletions() == diff_info.insertions() {
        if let Some(subject) = msg_info.subject() {
            if subject.to_ascii_lowercase().contains("rename") {
                classes.insert(CommitClass::RenameCommit);
            }
        }
    }

    classes
}

fn classify_commit(
    metadata: &CommitMetadata,
    diff_info: &DiffInfo,
    msg_info: &MessageInfo,
) -> CommitClasses {
    CommitClasses(do_classify_commit(metadata, diff_info, msg_info))
}

pub fn parse_commit<'repo>(
    commit: &'repo Commit<'repo>,
    repo: &'repo Repository,
) -> Result<CommitInfo<'repo>, Error> {
    let msg_info = commit.message().map(MessageInfo::new).unwrap_or_default();

    let metadata = CommitMetadata {
        id: commit.id().to_string(),
        author: commit.author().name().unwrap().to_string(),
        parents: commit.parent_count(),
    };

    if metadata.parents() >= 2 {
        return Ok(CommitInfo {
            metadata,
            diff_info: None,
            msg_info,
            classes: CommitClasses(EnumSet::from(CommitClass::MergeCommit)),
        });
    }

    let parent = commit.parents().next();

    let tree = commit.tree()?;
    let parent_tree = parent.as_ref().map(|p| p.tree()).transpose()?;

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
    let diff_stats = diff.stats()?;
    let diff_info = DiffInfo::new(&diff_stats);

    let classes = classify_commit(&metadata, &diff_info, &msg_info);

    Ok(CommitInfo {
        metadata,
        diff_info: Some(diff_info),
        msg_info,
        classes,
    })
}
