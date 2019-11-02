use enumset::{EnumSet, EnumSetType};
use regex::Regex;
use std::fmt::{Display, Formatter};

use crate::commit::{diff::DiffInfo, message::MessageInfo, metadata::CommitMetadata};

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
    pub fn classify_commit(
        metadata: &CommitMetadata,
        diff_info: &DiffInfo,
        msg_info: &MessageInfo,
    ) -> CommitClasses {
        CommitClasses(do_classify_commit(metadata, diff_info, msg_info))
    }

    pub fn from_set(classes: EnumSet<CommitClass>) -> CommitClasses {
        CommitClasses(classes)
    }

    pub fn as_set(self) -> EnumSet<CommitClass> {
        self.0
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

#[cfg(test)]
mod tests {
    use super::*;

    const COMMIT_ID: &str = "9335a4dc0e098830dec14fe3997c6a654695b935";

    lazy_static! {
        /// Ordinary commit metadata.
        static ref ORDINARY_META: CommitMetadata = {
            let id = COMMIT_ID.to_string();
            let author = "Leeroy Jenkins".to_string();
            let parents = 1;

            CommitMetadata::new(id, author, parents)
        };

        /// Initial commit metadata.
        static ref INITIAL_META: CommitMetadata = {
            let id = COMMIT_ID.to_string();
            let author = "Leeroy Jenkins".to_string();
            let parents = 0;

            CommitMetadata::new(id, author, parents)
        };

        /// Merge commit metadata. Parents number may be huge.
        static ref MERGE_META: CommitMetadata = {
            let id = COMMIT_ID.to_string();
            let author = "Leeroy Jenkins".to_string();
            let parents = 42;

            CommitMetadata::new(id, author, parents)
        };
    }

    #[test]
    fn empty_classes_are_rendered_as_empty_string() {
        let classes = CommitClasses(EnumSet::new());
        let rendered = format!("{}", classes);

        assert_eq!(rendered, "");
    }

    #[test]
    fn full_classes_set_is_rendered_correctly() {
        let mut classes_set = EnumSet::new();

        classes_set.insert(CommitClass::ShortCommit);
        classes_set.insert(CommitClass::MergeCommit);
        classes_set.insert(CommitClass::RefactorCommit);
        classes_set.insert(CommitClass::InitialCommit);

        let classes = CommitClasses(classes_set);
        let rendered = format!("{}", classes);

        // XXX: here we rely on the fact that EnumSet uses the order in which
        // variants are defined in enum. This behavior is consistent for
        // specific Rust/EnumSet versions, but may occasionally break after
        // updates, so keep in mind that this test is not perfect.
        assert_eq!(rendered, "MISR");
    }

    #[test]
    fn ordinary_commit_gets_no_special_classes() {
        let diff = DiffInfo::new(53, 102);
        let msg_info = msg_info_from_subject("Lorem ipsum dolor sit amet");

        let classes = do_classify_commit(&ORDINARY_META, &diff, &msg_info);

        assert!(classes.is_empty());
    }

    #[test]
    fn initial_commit_is_classified_when_no_parents() {
        let diff = DiffInfo::new(0, 0);
        let msg_info = msg_info_from_subject("Initial commit");

        let classes = do_classify_commit(&INITIAL_META, &diff, &msg_info);

        assert!(classes.contains(CommitClass::InitialCommit));
    }

    #[test]
    fn initial_commit_is_not_classified_when_parents_exist() {
        let diff = DiffInfo::new(0, 0);
        let diff2 = DiffInfo::new(42, 666);
        let msg_info = msg_info_from_subject("Initial commit");

        let classes = do_classify_commit(&ORDINARY_META, &diff, &msg_info);
        let classes2 = do_classify_commit(&ORDINARY_META, &diff2, &msg_info);
        let classes3 = do_classify_commit(&MERGE_META, &diff, &msg_info);

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
        MessageInfo::new(subject)
    }
}
