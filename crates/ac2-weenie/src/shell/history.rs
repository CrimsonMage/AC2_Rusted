//! The chat window's histories: the command line's (`CShell`'s `cmdHistory`, newest first)
//! and the names told and heard (`HistoryList`, package 0x1AB8: `CPlayer` slots 55 and 47,
//! which `CCommunicationSystem` 0xA and 0x2 create holding 50).

/// Which history the chat window walks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatHistory {
    /// The lines entered (`CWInterface::InqPrev/NextHistoryItem`).
    Command,
    /// The names told (`gmCWInterface::InqPrev/NextRetellHistoryItem`).
    Retell,
    /// The names heard from (`gmCWInterface::InqPrev/NextReplyHistoryItem`).
    Reply,
}

/// `CShell`'s command history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommandHistory {
    /// `cmdHistory`, newest first.
    items: Vec<String>,
    /// `currHistoryItem`.
    current: i32,
}

impl Default for CommandHistory {
    /// `CShell::CShell`: empty, `currHistoryItem` at `num - 1`.
    fn default() -> Self {
        Self {
            items: Vec::new(),
            current: -1,
        }
    }
}

impl CommandHistory {
    /// `CShell::AddCmdToHistory`: an earlier identical line (`wcscmp`) moves to the front and
    /// the walk restarts. `ResizeCmdHistory` only pops while the 20-line limit exceeds the
    /// count, after returning when the count is below it, so retail never trims the list.
    pub(crate) fn add(&mut self, line: &str) {
        if line.is_empty() {
            return;
        }
        self.current = -1;
        if let Some(at) = self.items.iter().position(|l| l == line) {
            self.items.remove(at);
        }
        self.items.insert(0, line.to_owned());
    }

    /// `CShell::InqHistoryItem`: one step older (`prev`) or newer, clamped to -1 ..
    /// `count - 1`.
    pub(crate) fn step(&mut self, prev: bool) -> Option<String> {
        let count = self.items.len() as i32;
        let mut at = self.current + if prev { 1 } else { -1 };
        if at < -1 {
            at = -1;
        } else if at >= count {
            at = count - 1;
        }
        self.current = at;
        usize::try_from(at)
            .ok()
            .and_then(|at| self.items.get(at))
            .cloned()
    }
}

/// `HistoryList`: names, newest first, at most `max`, and the walk's index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct HistoryList {
    /// Slot 0.
    items: Vec<String>,
    /// Slot 1.
    max: usize,
    /// Slot 2.
    index: i32,
}

impl Default for HistoryList {
    /// `HistoryList` 0xA, then 0x7 (50) as `CCommunicationSystem` 0x2 / 0xA make it.
    fn default() -> Self {
        Self {
            items: Vec::new(),
            max: 50,
            index: -1,
        }
    }
}

impl HistoryList {
    /// `HistoryList` 0x2: the index, clamped to -1 .. `count - 1`.
    fn set_index(&mut self, index: i32) {
        let last = self.items.len() as i32 - 1;
        self.index = if index < -1 { -1 } else { index.min(last) };
    }

    /// `HistoryList` 0xB: a name that is not already the newest (compared without case,
    /// 0x6) moves to the front (0x4 drops its first match); the list keeps `max` (0x7) and
    /// a walk in progress keeps pointing at the same name.
    pub(crate) fn add(&mut self, name: &str) {
        if self
            .items
            .first()
            .is_some_and(|front| front.eq_ignore_ascii_case(name))
        {
            return;
        }
        if let Some(at) = self.items.iter().position(|n| n.eq_ignore_ascii_case(name)) {
            self.items.remove(at);
        }
        self.items.insert(0, name.to_owned());
        self.items.truncate(self.max);
        if self.index != -1 {
            self.set_index(self.index + 1);
        }
    }

    /// `HistoryList` 0x8 (`prev`, older) or 0x3 (newer).
    pub(crate) fn step(&mut self, prev: bool) -> Option<String> {
        self.set_index(self.index + if prev { 1 } else { -1 });
        usize::try_from(self.index)
            .ok()
            .and_then(|at| self.items.get(at))
            .cloned()
    }

    /// `HistoryList` 0x1 (`ResetRetellHistoryIndex` / `ResetReplyHistoryIndex`).
    pub(crate) fn reset(&mut self) {
        self.index = -1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_command_history_walks_newest_first_and_clamps() {
        let mut h = CommandHistory::default();
        assert_eq!(h.step(true), None);
        for line in ["/a", "/b", "/a", ""] {
            h.add(line);
        }
        assert_eq!(h.step(true).as_deref(), Some("/a"));
        assert_eq!(h.step(true).as_deref(), Some("/b"));
        assert_eq!(h.step(true).as_deref(), Some("/b"));
        assert_eq!(h.step(false).as_deref(), Some("/a"));
        assert_eq!(h.step(false), None);
        assert_eq!(h.step(false), None);
        assert_eq!(h.step(true).as_deref(), Some("/a"));
        // Never trimmed.
        for i in 0..30 {
            h.add(&format!("/{i}"));
        }
        assert_eq!(h.items.len(), 32);
    }

    #[test]
    fn name_histories_keep_fifty_and_follow_the_walk() {
        let mut h = HistoryList::default();
        h.add("Birch");
        h.add("BIRCH");
        h.add("Oak");
        assert_eq!(h.step(true).as_deref(), Some("Oak"));
        // A new name while walking: the index moves with "Oak".
        h.add("Ash");
        assert_eq!(h.step(true).as_deref(), Some("Birch"));
        h.add("oak");
        assert_eq!(h.items, ["oak", "Ash", "Birch"]);
        h.reset();
        assert_eq!(h.step(false), None);
        for i in 0..60 {
            h.add(&i.to_string());
        }
        assert_eq!(h.items.len(), 50);
    }
}
