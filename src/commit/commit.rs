use enumset::EnumSet;

use crate::commit::{
    class::{Class, Classes},
    diff::DiffInfo,
    message::MessageInfo,
    metadata::Metadata,
};

/// A parsed and classified commit with all the data
/// required for scoring.
pub struct Commit {
    metadata: Metadata,
    diff_info: Option<DiffInfo>,
    msg_info: MessageInfo,
    classes: Classes,
}

impl Commit {
    pub fn new(metadata: Metadata, diff_info: DiffInfo, msg_info: MessageInfo) -> Self {
        let classes = Classes::classify_commit(&metadata, &diff_info, &msg_info);

        Self {
            metadata,
            diff_info: Some(diff_info),
            msg_info,
            classes,
        }
    }

    pub fn new_from_merge(metadata: Metadata, msg_info: MessageInfo) -> Self {
        let classes = Classes::from_set(EnumSet::from(Class::Merge));

        Self {
            metadata,
            diff_info: None,
            msg_info,
            classes,
        }
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn diff_info(&self) -> &Option<DiffInfo> {
        &self.diff_info
    }

    pub fn msg_info(&self) -> &MessageInfo {
        &self.msg_info
    }

    pub fn classes(&self) -> Classes {
        self.classes
    }
}
