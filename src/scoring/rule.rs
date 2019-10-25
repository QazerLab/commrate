use crate::commit::{CommitClass, CommitInfo};

use enumset::EnumSet;

/// Scoring rule takes care about the specific aspect of the
/// commit quality and returns result from 0 to 1 depending on
/// how good the commit is.
///
/// Note, that the rule itself does not care about
///
/// * what will be its weight in the overall score;
/// * what is the real scale of the score.
///
/// Both of these items are addressed at the higher levels.
pub trait Rule {
    /// Check the commit against this rule and return the result
    /// between 0 and 1 depending on the commit quality.
    fn score(&self, commit: &CommitInfo) -> f32;
}

/// This rule checks the commit subject (the first message line),
/// which must be
///
/// * present;
/// * long enough ("fix" or "refactoring" is a bad subject);
/// * not too long (does not play well with things like log --oneline).
///
/// This is pretty crucial, as the subject is inspected much more
/// frequently than the rest of the body. However, no stylistical
/// limitations are imposed - only length is scored.
pub struct SubjectRule;

impl Rule for SubjectRule {
    fn score(&self, commit: &CommitInfo) -> f32 {
        let classes = commit.classes().as_set();

        // Typical "Initial commit" gets penalized by ordinary rules,
        // let's forgive this short but traditional message.
        if classes.contains(CommitClass::InitialCommit) {
            return 1.0;
        }

        let subject = commit.msg_info().subject().unwrap_or("");

        // This is a special case for ugly commits, which specify
        // a ticket/issue ID as commit subject. These are long
        // enough to get over 10 chars, but should not get even
        // a single score point.
        //
        // Not a bulletproof, but cuts the most obvious crap.
        if subject.split_ascii_whitespace().count() <= 1 {
            return 0.0;
        }

        let len = subject.len();

        match len {
            0..=10 => 0.0,

            // Smoothly ascend to more or less reasonable length (and score).
            11..=20 => (len as f32 - 10.0) / 10.0,

            // The optimal length: long enough to be meaningful and
            // short enough to fit oneline log or e-mailed patch.
            21..=70 => 1.0,

            // The descending branch of the function goes much more smoothly.
            // Though long subjects are not good, they at least carry some
            // useful information. Let's not be so radical here.
            71..=100 => (100.0 - len as f32) / 100.0,

            // 100+ chars in subject deserve no mercy, really.
            _ => 0.0,
        }
    }
}

/// This rule checks that the commit has at least *any* body.
///
/// Special commits classes are not penalized for body absence.
pub struct BodyPresenceRule;

impl Rule for BodyPresenceRule {
    fn score(&self, commit: &CommitInfo) -> f32 {
        if commit.msg_info().body_len() > 0 || commit_is_special(commit) {
            1.0
        } else {
            0.0
        }
    }
}

/// This rule ensures that the subject and the body are in
/// different paragraphs, i.e. if the body is present, it
/// starts on line 3, and line 2 is the empty line.
///
/// This makes the message more readable. Absence of the break
/// between the subject and the body results in the penalty.
///
/// In fact, this rule also penalizes non-special commits
/// without the body at all, and this is not a bug.
pub struct SubjectBodyBreakRule;

impl Rule for SubjectBodyBreakRule {
    fn score(&self, commit: &CommitInfo) -> f32 {
        let msg_info = commit.msg_info();

        if msg_info.body_len() > 0 {
            if msg_info.break_after_subject() {
                1.0
            } else {
                0.0
            }
        } else {
            if commit_is_special(commit) {
                1.0
            } else {
                0.0
            }
        }
    }
}

/// This rule estimates the relation of the message body length
/// and the diff size.
///
/// In general, then longer the diff, the better explanation
/// should it have. However, the dependency here is clearly
/// non-linear. Also, there are obvious exceptions for special
/// cases, which should not be penalized for short/absent body.
pub struct BodyLenRule;

impl Rule for BodyLenRule {
    fn score(&self, commit: &CommitInfo) -> f32 {
        if commit_is_special(commit) {
            return 1.0;
        }

        let diff_option = commit.diff_info();
        if diff_option.is_none() {
            // XXX: this may happen only for merge commits,
            // which should not reach here.
            // Maybe panic instead of graceful return?
            return 1.0;
        }

        let diff_size = diff_option.as_ref().unwrap().diff_total();
        let body_len = commit.msg_info().body_len();

        // This formula if VERY rough and thus probably should be adjusted
        // with coefficients, especially in low diff size or low body len areas.
        //
        // XXX: +1.0 is to pull ln() value for empty body to zero.
        let score = (body_len as f32 + 1.0).ln() / (diff_size as f32).ln();

        // To reach this maximum, there should be approximately
        //
        // * one line body for barely long diff (SHORT_COMMIT_SIZE + few lines);
        // * 3-4 lines of body for medium diff (~250 lines);
        // * few paragraphs of body for large diff (500-1000 lines).
        //
        // For larger diffs, the maximum is almost unreachable, unless the author
        // is insane and writes an essay in the log.
        if score > 1.0 {
            1.0
        } else {
            score
        }
    }
}

/// This rule checks the commit message for being well-wrapped.
///
/// Wrapping the message body lines to a reasonable length is a good tone.
/// However, some things may come unwrapped pretty legally, e.g. some
/// copy-pasted log or pesudo-graphics (ASCII diagrams, etc.).
///
/// This rule provides a score based on the fraction of unwrapped lines
/// in the whole message body. Even if the message contains few lines of
/// ordinary text and a medium-sized unwrapped copy-paste, this rule
/// still will grant the commit some score.
///
/// If everything else is OK, the overall score will be high enough to
/// reach the highest grade.
pub struct BodyWrappingRule;

impl Rule for BodyWrappingRule {
    fn score(&self, commit: &CommitInfo) -> f32 {
        let msg_info = commit.msg_info();
        let body_lines = msg_info.body_lines();

        if msg_info.body_lines() == 0 {
            if commit_is_special(commit) {
                return 1.0;
            } else {
                return 0.0;
            }
        }

        let lines_unwrapped = msg_info.body_unwrapped_lines();

        1.0 - lines_unwrapped as f32 / body_lines as f32
    }
}

/// This rule grants some additional score for having well-known
/// metadata lines in the commit message.
///
/// This stuff is optional in most projects, but is a good practice,
/// so this rule is expected to have very low weight. Consider
/// it as a little bonus which may raise the grade when the score
/// is close to the boudary between different grades.
pub struct MetadataLinesRule;

impl Rule for MetadataLinesRule {
    fn score(&self, commit: &CommitInfo) -> f32 {
        match commit.msg_info().metadata_lines() {
            0 => 0.0,
            1 => 0.6,
            2 => 0.8,
            _ => 1.0,
        }
    }
}

fn commit_is_special(commit: &CommitInfo) -> bool {
    let classes = commit.classes().as_set();

    !classes.intersection(*SPECIAL_CLASSES).is_empty()
}

// Commits of some classes are scored in relaxed fashion.
// Most rules use the same set of such special classes,
// so let's predefine this set here.
lazy_static! {
    static ref SPECIAL_CLASSES: EnumSet<CommitClass> = {
        let mut special_set = EnumSet::new();

        special_set.insert(CommitClass::ShortCommit);
        special_set.insert(CommitClass::RefactorCommit);
        special_set.insert(CommitClass::InitialCommit);

        special_set
    };
}
