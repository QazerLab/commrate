use std::collections::HashSet;

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
    // TODO: test message info parsing.
}
