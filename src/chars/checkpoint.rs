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

    /// Return true if the given string matches the head of the iterator.
    ///
    /// Important: This method will advance the iterator by `s.len()` *before*
    /// checking so characters will be lost if this method returns false. This
    /// method is only suitable if the next action after recieving false is to
    /// abort the checkpoint.
    ///
    /// ```
    /// # use span::Chars;
    /// let mut chars = Chars::new("123456");
    ///
    /// let mut checkpoint = chars.checkpoint();
    /// assert!(!checkpoint.head_matches("1234567"));
    /// assert_eq!(checkpoint.next(), None);
    /// checkpoint.abort();
    ///
    /// let mut checkpoint = chars.checkpoint();
    /// assert!(!checkpoint.head_matches("1238"));
    /// assert_eq!(checkpoint.next(), Some('5'));
    /// checkpoint.abort();
    ///
    /// let mut checkpoint = chars.checkpoint();
    /// assert!(checkpoint.head_matches("1234"));
    /// assert_eq!(checkpoint.next(), Some('5'));
    /// ```
    pub fn head_matches(&mut self, s: &str) -> bool {
        let head = self.take(s.len()).collect::<String>();
        s == head
    }

    /// Lookahead at the next item in the iterator without advancing. Peek
    /// always returns the same value until a call to next.
    ///
    /// If the checkpoint is committed the peeked character will not be removed
    /// from the underlying iterator
    ///
    /// ```
    /// # use span::Chars;
    /// let mut chars = Chars::new("123456");
    /// let mut checkpoint = chars.checkpoint();
    /// assert_eq!(checkpoint.next(), Some('1'));
    /// assert_eq!(checkpoint.peek(), Some('2'));
    /// assert_eq!(checkpoint.peek(), Some('2'));
    /// assert_eq!(checkpoint.next(), Some('2'));
    /// assert_eq!(checkpoint.peek(), Some('3'));
    /// checkpoint.commit();
    /// assert_eq!(chars.next(), Some('3'));
    /// ```
    pub fn peek(&mut self) -> Option<char> {
        self.chars.it.peek_nth(self.peeked).copied()
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
