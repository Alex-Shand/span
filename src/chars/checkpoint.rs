use super::Chars;

/// See [Chars::checkpoint]
#[expect(missing_debug_implementations)]
pub struct Checkpoint<'a> {
    chars: &'a mut Chars,
    peeked: usize,
}

impl<'a> Checkpoint<'a> {
    pub(crate) fn new(chars: &'a mut Chars) -> Self {
        Self { chars, peeked: 0 }
    }

    /// Releases the underlying [Chars] iterator with no changes. Identical to
    /// dropping it
    pub fn abort(self) {}

    /// Commits the checkpoint by advancing the underlying [Chars] iterator
    /// across all of the characters returned by the checkpoint
    pub fn commit(self) {
        for _ in self.chars.take(self.peeked) {}
    }
}

impl Iterator for Checkpoint<'_> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.chars.it.peek_nth(self.peeked).copied()?;
        self.peeked += 1;
        Some(result)
    }
}
