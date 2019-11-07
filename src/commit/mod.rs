mod class;
pub use class::{Class, Classes};

#[allow(clippy::module_inception)]
mod commit;
pub use commit::Commit;

mod diff;
pub use diff::DiffInfo;

mod message;
pub use message::MessageInfo;

mod metadata;
pub use metadata::Metadata;
