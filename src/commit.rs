use enumset::{EnumSet, EnumSetType};
use regex::Regex;
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

/// A parsed and classified commit with all the data
/// required for scoring.
pub struct CommitInfo {
    metadata: CommitMetadata,
    diff_info: Option<DiffInfo>,
    msg_info: MessageInfo,
    classes: CommitClasses,
}

impl CommitInfo {
    pub fn new(metadata: CommitMetadata, diff_info: DiffInfo, msg_info: MessageInfo) -> CommitInfo {
        let classes = classify_commit(&metadata, &diff_info, &msg_info);

        CommitInfo {
            metadata,
            diff_info: Some(diff_info),
            msg_info,
            classes,
        }
    }

    pub fn new_from_merge(metadata: CommitMetadata, msg_info: MessageInfo) -> CommitInfo {
        let classes = CommitClasses(EnumSet::from(CommitClass::MergeCommit));

        CommitInfo {
            metadata,
            diff_info: None,
            msg_info,
            classes,
        }
    }

    pub fn metadata(&self) -> &CommitMetadata {
        &self.metadata
    }

    pub fn diff_info(&self) -> &Option<DiffInfo> {
        &self.diff_info
    }

    pub fn msg_info(&self) -> &MessageInfo {
        &self.msg_info
    }

    pub fn classes(&self) -> CommitClasses {
        self.classes
    }
}

/// A commit metadata, which is easy to obtain from
/// the repository without any heavy processing.
///
/// XXX: do we really need owned Strings here, or there
/// is a way to decoupe from git2 Oid's pecularities?
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

/// `MessageInfo` contains the metrics obtained from
/// the commit message for scoring.
#[derive(Default, Debug)]
pub struct MessageInfo {
    subject: Option<String>,
    break_after_subject: bool,
    body_len: usize,
    body_lines: usize,
    body_unwrapped_lines: usize,
    metadata_lines: usize,
}

impl MessageInfo {
    pub fn new(raw_message: &str) -> MessageInfo {
        let mut subject: Option<String> = None;
        let mut break_after_subject = false;
        let mut body_len = 0;
        let mut body_lines = 0;
        let mut body_unwrapped_lines = 0;
        let mut metadata_lines = 0;

        // Here we rely on line numbers, as Git strips
        // leading and trailing empty lines during commit.
        // This means, that the subject is always line 0.
        for (line_num, line) in raw_message.lines().enumerate() {
            if line_num == 0 {
                // XXX: we need an owned string here for being able to
                // conventently pass the MessageInfo out of intermediate
                // iterator items.
                //
                // TODO: try to find the way to use a reference without
                // giving up convenient iterators over commits.
                subject = Some(line.to_string());
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
        self.subject.as_ref().map(|ref s| s.as_str())
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

/// Maximum diff size (lines total) for short commits.
pub const SHORT_COMMIT_LENGTH: usize = 25;

/// For refactoring commits, we allow a slight difference between
/// insertions and deletions (5% of total diff) to ensure
/// that move-related things like fixing imports and so on
/// do not subvert the correct classification of these commits.
pub const REFACTOR_COMMIT_ALLOWED_DIFF: f32 = 0.05;

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
    /// or piece of code (e.g. function) or moving this
    /// piece to another file usually do not require
    /// additional explanations and may be described with a
    /// single subject line, e.g.
    ///
    /// "Rename Foo::bar() to Foo::baz()"
    /// "Rename src/module to src/another_module"
    /// "Move redhat::Openshift to ibm::Openshift"
    ///
    /// Such commits could be pretty long though, so they
    /// require special treatment.
    RefactorCommit,
}

/// A newtype wrapper for implementing Display.
#[derive(Clone, Copy, Debug)]
pub struct CommitClasses(EnumSet<CommitClass>);

impl Display for CommitClasses {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let set_len = self.as_set().len();
        let mut buf = String::with_capacity(set_len);
        for class in self.0 {
            buf.push(match class {
                CommitClass::MergeCommit => 'M',
                CommitClass::InitialCommit => 'I',
                CommitClass::RefactorCommit => 'R',
                CommitClass::ShortCommit => 'S',
            });
        }

        Display::fmt(&buf, f)
    }
}

impl CommitClasses {
    pub fn as_set(self) -> EnumSet<CommitClass> {
        self.0
    }
}

fn classify_commit(
    metadata: &CommitMetadata,
    diff_info: &DiffInfo,
    msg_info: &MessageInfo,
) -> CommitClasses {
    CommitClasses(do_classify_commit(metadata, diff_info, msg_info))
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
    let allowed_diff = (diff_info.diff_total() as f32 * REFACTOR_COMMIT_ALLOWED_DIFF) as isize;
    let actual_diff = (diff_info.deletions() as isize - diff_info.insertions() as isize).abs();
    if actual_diff <= allowed_diff {
        if let Some(subject) = msg_info.subject() {
            let regex = Regex::new(r#"(?i)(\bmoved?\b)|(\brenamed?\b)"#).unwrap();
            if regex.is_match(subject) {
                classes.insert(CommitClass::RefactorCommit);
            }
        }
    }

    classes
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

#[cfg(test)]
mod tests {
    use super::*;

    const COMMIT_ID: &str = "9335a4dc0e098830dec14fe3997c6a654695b935";

    lazy_static! {
        static ref ORDINARY_META: CommitMetadata = {
            CommitMetadata {
                id: COMMIT_ID.to_string(),
                author: "Leeroy Jenkins".to_string(),
                parents: 1,
            }
        };
    }

    #[test]
    fn initial_commit_is_classified_when_no_parents() {
        let meta = CommitMetadata {
            id: COMMIT_ID.to_string(),
            author: "Leeroy Jenkins".to_string(),
            parents: 0,
        };

        let diff = DiffInfo::new(0, 0);
        let msg_info = msg_info_from_subject("Initial commit");

        let classes = do_classify_commit(&meta, &diff, &msg_info);

        assert!(classes.contains(CommitClass::InitialCommit));
    }

    #[test]
    fn initial_commit_is_not_classified_when_parents_exist() {
        let meta = CommitMetadata {
            id: COMMIT_ID.to_string(),
            author: "Leeroy Jenkins".to_string(),
            parents: 1,
        };

        let meta2 = CommitMetadata {
            id: COMMIT_ID.to_string(),
            author: "Leeroy Jenkins".to_string(),
            parents: 42,
        };

        let diff = DiffInfo::new(0, 0);
        let diff2 = DiffInfo::new(42, 666);
        let msg_info = msg_info_from_subject("Initial commit");

        let classes = do_classify_commit(&meta, &diff, &msg_info);
        let classes2 = do_classify_commit(&meta, &diff2, &msg_info);
        let classes3 = do_classify_commit(&meta2, &diff, &msg_info);

        assert!(!classes.contains(CommitClass::InitialCommit));
        assert!(!classes2.contains(CommitClass::InitialCommit));
        assert!(!classes3.contains(CommitClass::InitialCommit));
    }

    #[test]
    fn short_commit_is_classified_for_single_line_diff() {
        let diff = DiffInfo::new(1, 0);
        let msg_info = msg_info_from_subject("Fix NPE in CustomMetricsController");

        let classes = do_classify_commit(&ORDINARY_META, &diff, &msg_info);

        assert!(classes.contains(CommitClass::ShortCommit));
    }

    #[test]
    fn short_commit_is_not_classified_for_huge_diff() {
        let diff = DiffInfo::new(666, 42);
        let msg_info = msg_info_from_subject("Fix NPE in CustomMetricsController");

        let classes = do_classify_commit(&ORDINARY_META, &diff, &msg_info);

        assert!(!classes.contains(CommitClass::ShortCommit));
    }

    #[test]
    fn refactor_commit_is_classified_with_infinitive() {
        let diff = DiffInfo::new(42, 42);
        let msg_info = msg_info_from_subject("move Snowden to Russia");
        let msg_info2 = msg_info_from_subject("rename C# to Java");

        let classes = do_classify_commit(&ORDINARY_META, &diff, &msg_info);
        let classes2 = do_classify_commit(&ORDINARY_META, &diff, &msg_info2);

        assert!(classes.contains(CommitClass::RefactorCommit));
        assert!(classes2.contains(CommitClass::RefactorCommit));
    }

    #[test]
    fn refactor_commit_is_classified_with_past() {
        let diff = DiffInfo::new(42, 42);
        let msg_info = msg_info_from_subject("moved Snowden to Russia");
        let msg_info2 = msg_info_from_subject("renamed C# to Java");

        let classes = do_classify_commit(&ORDINARY_META, &diff, &msg_info);
        let classes2 = do_classify_commit(&ORDINARY_META, &diff, &msg_info2);

        assert!(classes.contains(CommitClass::RefactorCommit));
        assert!(classes2.contains(CommitClass::RefactorCommit));
    }

    #[test]
    fn refactor_commit_is_classified_with_mixed_case() {
        let diff = DiffInfo::new(42, 42);
        let msg_info = msg_info_from_subject("MoVe Snowden to Russia");
        let msg_info2 = msg_info_from_subject("ReNaMe C# to Java");

        let classes = do_classify_commit(&ORDINARY_META, &diff, &msg_info);
        let classes2 = do_classify_commit(&ORDINARY_META, &diff, &msg_info2);

        assert!(classes.contains(CommitClass::RefactorCommit));
        assert!(classes2.contains(CommitClass::RefactorCommit));
    }

    #[test]
    fn refactor_commit_is_classified_with_keywords_in_middle() {
        let diff = DiffInfo::new(42, 42);
        let msg_info = msg_info_from_subject("I moved Snowden to Russia");
        let msg_info2 = msg_info_from_subject("I renamed C# to Java");

        let classes = do_classify_commit(&ORDINARY_META, &diff, &msg_info);
        let classes2 = do_classify_commit(&ORDINARY_META, &diff, &msg_info2);

        assert!(classes.contains(CommitClass::RefactorCommit));
        assert!(classes2.contains(CommitClass::RefactorCommit));
    }

    #[test]
    fn refactor_commit_is_classified_with_small_ins_del_diff() {
        let diff = DiffInfo::new(50, 52);
        let msg_info = msg_info_from_subject("Move Snowden to Russia");
        let msg_info2 = msg_info_from_subject("Rename C# to Java");

        let classes = do_classify_commit(&ORDINARY_META, &diff, &msg_info);
        let classes2 = do_classify_commit(&ORDINARY_META, &diff, &msg_info2);

        assert!(classes.contains(CommitClass::RefactorCommit));
        assert!(classes2.contains(CommitClass::RefactorCommit));
    }

    #[test]
    fn refactor_commit_is_not_classified_without_keywords() {
        let diff = DiffInfo::new(42, 42);
        let msg_info = msg_info_from_subject("Improve character movement rendering");
        let msg_info2 = msg_info_from_subject("Just for lulz bro");

        let classes = do_classify_commit(&ORDINARY_META, &diff, &msg_info);
        let classes2 = do_classify_commit(&ORDINARY_META, &diff, &msg_info2);

        assert!(!classes.contains(CommitClass::RefactorCommit));
        assert!(!classes2.contains(CommitClass::RefactorCommit));
    }

    #[test]
    fn refactor_commit_is_not_classified_with_large_ins_del_diff() {
        let diff = DiffInfo::new(10, 500);
        let msg_info = msg_info_from_subject("Move Snowden to Russia");
        let msg_info2 = msg_info_from_subject("Rename C# to Java");

        let classes = do_classify_commit(&ORDINARY_META, &diff, &msg_info);
        let classes2 = do_classify_commit(&ORDINARY_META, &diff, &msg_info2);

        assert!(!classes.contains(CommitClass::RefactorCommit));
        assert!(!classes2.contains(CommitClass::RefactorCommit));
    }

    fn msg_info_from_subject(subject: &str) -> MessageInfo {
        MessageInfo {
            subject: Some(subject.to_string()),
            break_after_subject: false,
            body_len: 0,
            body_lines: 0,
            body_unwrapped_lines: 0,
            metadata_lines: 0,
        }
    }
}
