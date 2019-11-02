use enumset::{EnumSet, EnumSetType};
use regex::Regex;
use std::fmt::{Display, Formatter};

use crate::commit::{diff::DiffInfo, message::MessageInfo, metadata::Metadata};

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
pub enum Class {
    Merge,

    /// Initial commits usually do not have anything else
    /// a subject "Initial commit" in the message, though
    /// they frequently have huge diff.
    Initial,

    /// Short commits may contain some primitive change
    /// which does not require additional explanations:
    /// version bumps, typo fixes, etc.
    ///
    /// No penalty for message body should be applied to
    /// such commits.
    Short,

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
    Refactor,
}

/// A newtype wrapper for implementing Display.
#[derive(Clone, Copy, Debug)]
pub struct Classes(EnumSet<Class>);

impl Display for Classes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let set_len = self.as_set().len();
        let mut buf = String::with_capacity(set_len);
        for class in self.0 {
            buf.push(match class {
                Class::Merge => 'M',
                Class::Initial => 'I',
                Class::Refactor => 'R',
                Class::Short => 'S',
            });
        }

        Display::fmt(&buf, f)
    }
}

impl Classes {
    pub fn classify_commit(
        metadata: &Metadata,
        diff_info: &DiffInfo,
        msg_info: &MessageInfo,
    ) -> Self {
        Self(classify(metadata, diff_info, msg_info))
    }

    pub fn from_set(classes: EnumSet<Class>) -> Self {
        Self(classes)
    }

    pub fn as_set(self) -> EnumSet<Class> {
        self.0
    }
}

fn classify(metadata: &Metadata, diff_info: &DiffInfo, msg_info: &MessageInfo) -> EnumSet<Class> {
    let mut classes = EnumSet::new();

    if metadata.parents() == 0 {
        classes.insert(Class::Initial);
    }

    if diff_info.diff_total() < SHORT_COMMIT_LENGTH {
        classes.insert(Class::Short);
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
                classes.insert(Class::Refactor);
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
        static ref ORDINARY_META: Metadata = {
            let id = COMMIT_ID.to_string();
            let author = "Leeroy Jenkins".to_string();
            let parents = 1;

            Metadata::new(id, author, parents)
        };

        /// Initial commit metadata.
        static ref INITIAL_META: Metadata = {
            let id = COMMIT_ID.to_string();
            let author = "Leeroy Jenkins".to_string();
            let parents = 0;

            Metadata::new(id, author, parents)
        };

        /// Merge commit metadata. Parents number may be huge.
        static ref MERGE_META: Metadata = {
            let id = COMMIT_ID.to_string();
            let author = "Leeroy Jenkins".to_string();
            let parents = 42;

            Metadata::new(id, author, parents)
        };
    }

    #[test]
    fn empty_classes_are_rendered_as_empty_string() {
        let classes = Classes(EnumSet::new());
        let rendered = format!("{}", classes);

        assert_eq!(rendered, "");
    }

    #[test]
    fn full_classes_set_is_rendered_correctly() {
        let mut classes_set = EnumSet::new();

        classes_set.insert(Class::Short);
        classes_set.insert(Class::Merge);
        classes_set.insert(Class::Refactor);
        classes_set.insert(Class::Initial);

        let classes = Classes(classes_set);
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
        let msg_info = MessageInfo::new("Lorem ipsum dolor sit amet");

        let classes = classify(&ORDINARY_META, &diff, &msg_info);

        assert!(classes.is_empty());
    }

    #[test]
    fn initial_commit_is_classified_when_no_parents() {
        let diff = DiffInfo::new(0, 0);
        let msg_info = MessageInfo::new("Initial commit");

        let classes = classify(&INITIAL_META, &diff, &msg_info);

        assert!(classes.contains(Class::Initial));
    }

    #[test]
    fn initial_commit_is_not_classified_when_parents_exist() {
        let diff = DiffInfo::new(0, 0);
        let diff2 = DiffInfo::new(42, 666);
        let msg_info = MessageInfo::new("Initial commit");

        let classes = classify(&ORDINARY_META, &diff, &msg_info);
        let classes2 = classify(&ORDINARY_META, &diff2, &msg_info);
        let classes3 = classify(&MERGE_META, &diff, &msg_info);

        assert!(!classes.contains(Class::Initial));
        assert!(!classes2.contains(Class::Initial));
        assert!(!classes3.contains(Class::Initial));
    }

    #[test]
    fn short_commit_is_classified_for_single_line_diff() {
        let diff = DiffInfo::new(1, 0);
        let msg_info = MessageInfo::new("Fix NPE in CustomMetricsController");

        let classes = classify(&ORDINARY_META, &diff, &msg_info);

        assert!(classes.contains(Class::Short));
    }

    #[test]
    fn short_commit_is_not_classified_for_huge_diff() {
        let diff = DiffInfo::new(666, 42);
        let msg_info = MessageInfo::new("Fix NPE in CustomMetricsController");

        let classes = classify(&ORDINARY_META, &diff, &msg_info);

        assert!(!classes.contains(Class::Short));
    }

    #[test]
    fn refactor_commit_is_classified_with_infinitive() {
        let diff = DiffInfo::new(42, 42);
        let msg_info = MessageInfo::new("move Snowden to Russia");
        let msg_info2 = MessageInfo::new("rename C# to Java");

        let classes = classify(&ORDINARY_META, &diff, &msg_info);
        let classes2 = classify(&ORDINARY_META, &diff, &msg_info2);

        assert!(classes.contains(Class::Refactor));
        assert!(classes2.contains(Class::Refactor));
    }

    #[test]
    fn refactor_commit_is_classified_with_past() {
        let diff = DiffInfo::new(42, 42);
        let msg_info = MessageInfo::new("moved Snowden to Russia");
        let msg_info2 = MessageInfo::new("renamed C# to Java");

        let classes = classify(&ORDINARY_META, &diff, &msg_info);
        let classes2 = classify(&ORDINARY_META, &diff, &msg_info2);

        assert!(classes.contains(Class::Refactor));
        assert!(classes2.contains(Class::Refactor));
    }

    #[test]
    fn refactor_commit_is_classified_with_mixed_case() {
        let diff = DiffInfo::new(42, 42);
        let msg_info = MessageInfo::new("MoVe Snowden to Russia");
        let msg_info2 = MessageInfo::new("ReNaMe C# to Java");

        let classes = classify(&ORDINARY_META, &diff, &msg_info);
        let classes2 = classify(&ORDINARY_META, &diff, &msg_info2);

        assert!(classes.contains(Class::Refactor));
        assert!(classes2.contains(Class::Refactor));
    }

    #[test]
    fn refactor_commit_is_classified_with_keywords_in_middle() {
        let diff = DiffInfo::new(42, 42);
        let msg_info = MessageInfo::new("I moved Snowden to Russia");
        let msg_info2 = MessageInfo::new("I renamed C# to Java");

        let classes = classify(&ORDINARY_META, &diff, &msg_info);
        let classes2 = classify(&ORDINARY_META, &diff, &msg_info2);

        assert!(classes.contains(Class::Refactor));
        assert!(classes2.contains(Class::Refactor));
    }

    #[test]
    fn refactor_commit_is_classified_with_small_ins_del_diff() {
        let diff = DiffInfo::new(50, 52);
        let msg_info = MessageInfo::new("Move Snowden to Russia");
        let msg_info2 = MessageInfo::new("Rename C# to Java");

        let classes = classify(&ORDINARY_META, &diff, &msg_info);
        let classes2 = classify(&ORDINARY_META, &diff, &msg_info2);

        assert!(classes.contains(Class::Refactor));
        assert!(classes2.contains(Class::Refactor));
    }

    #[test]
    fn refactor_commit_is_not_classified_without_keywords() {
        let diff = DiffInfo::new(42, 42);
        let msg_info = MessageInfo::new("Improve character movement rendering");
        let msg_info2 = MessageInfo::new("Just for lulz bro");

        let classes = classify(&ORDINARY_META, &diff, &msg_info);
        let classes2 = classify(&ORDINARY_META, &diff, &msg_info2);

        assert!(!classes.contains(Class::Refactor));
        assert!(!classes2.contains(Class::Refactor));
    }

    #[test]
    fn refactor_commit_is_not_classified_with_large_ins_del_diff() {
        let diff = DiffInfo::new(10, 500);
        let msg_info = MessageInfo::new("Move Snowden to Russia");
        let msg_info2 = MessageInfo::new("Rename C# to Java");

        let classes = classify(&ORDINARY_META, &diff, &msg_info);
        let classes2 = classify(&ORDINARY_META, &diff, &msg_info2);

        assert!(!classes.contains(Class::Refactor));
        assert!(!classes2.contains(Class::Refactor));
    }
}
