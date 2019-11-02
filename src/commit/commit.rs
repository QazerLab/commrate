use enumset::EnumSet;

use crate::commit::{
    class::{CommitClass, CommitClasses},
    diff::DiffInfo,
    message::MessageInfo,
    metadata::CommitMetadata,
};

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
        let classes = CommitClasses::classify_commit(&metadata, &diff_info, &msg_info);

        CommitInfo {
            metadata,
            diff_info: Some(diff_info),
            msg_info,
            classes,
        }
    }

    pub fn new_from_merge(metadata: CommitMetadata, msg_info: MessageInfo) -> CommitInfo {
        let classes = CommitClasses::from_set(EnumSet::from(CommitClass::MergeCommit));

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
