use itertools::{Itertools as _, PeekNth, PeekingNext};
use owned_chars::OwnedCharsExt;

use crate::{AbsoluteSpan, LineAndColumn, RelativeSpan, Span};

mod checkpoint;
pub use self::checkpoint::Checkpoint;

/// TokenHandle
#[expect(missing_copy_implementations, missing_debug_implementations)]
pub struct TokenHandle(Position);

#[derive(Copy, Clone)]
struct Position {
    loc: usize,
    line: usize,
    col: usize,
}

/// Character iterator that automatically tracks line and column location
/// The spans yielded by Chars uses 0 based indexing for absolute byte positions
/// and 1 based indexing for relative indexing
///
/// The start_token and end_token methods are used to generate token spans
/// pointing at ranges in the input
/// ```
/// # use span::*;
/// let mut chars = Chars::new("123456");
/// let start1 = chars.start_token();
/// assert_eq!(chars.next(), Some('1'));
/// assert_eq!(chars.next(), Some('2'));
/// let start2 = chars.start_token();
/// assert_eq!(chars.next(), Some('3'));
/// let span1 = chars.end_token(start1);
/// assert_eq!(chars.next(), Some('4'));
/// assert_eq!(chars.next(), Some('5'));
/// assert_eq!(chars.next(), Some('6'));
/// let span2 = chars.end_token(start2);
/// assert_eq!(format!("{span1:#}"), "line 1 column 1 to column 4");
/// assert_eq!(format!("{span2:#}"), "line 1 column 3 to column 7")
/// ```
#[allow(missing_debug_implementations)]
pub struct Chars {
    it: PeekNth<Box<dyn Iterator<Item = char>>>,
    current: Position,
}

impl Chars {
    /// Constructor
    #[must_use]
    pub fn new(str: impl Into<String>) -> Self {
        let it: Box<dyn Iterator<Item = char>> =
            Box::new(OwnedCharsExt::into_chars(str.into()));
        Self {
            it: itertools::peek_nth(it),
            current: Position {
                loc: 0,
                line: 1,
                col: 1,
            },
        }
    }

    /// Lookahead at the next item in the iterator without advancing. Peek
    /// always returns the same value until a call to next.
    ///
    /// ```
    /// # use span::*;
    /// let mut chars = Chars::new("1234");
    /// assert_eq!(chars.peek(), Some('1'));
    /// assert_eq!(chars.peek(), Some('1'));
    /// assert_eq!(chars.next(), Some('1'));
    /// assert_eq!(chars.peek(), Some('2'));
    /// assert_eq!(chars.peek(), Some('2'));
    /// ```
    pub fn peek(&mut self) -> Option<char> {
        self.it.peek().copied()
    }

    /// take_while except it only advances the iterator _after_ the test returns
    /// true
    ///
    /// ```
    /// # use span::*;
    /// let mut chars = Chars::new("111222");
    /// let ones = chars.peek_while(|c| c == '1').collect::<String>();
    /// let twos = chars.collect::<String>();
    /// assert_eq!(ones, "111");
    /// assert_eq!(twos, "222");
    /// ```
    pub fn peek_while<'a>(
        &'a mut self,
        test: impl Fn(char) -> bool + 'a,
    ) -> impl Iterator<Item = char> + 'a {
        self.peeking_take_while(move |c| test(*c))
    }

    /// Mark the beginning of a token
    #[must_use]
    pub fn start_token(&self) -> TokenHandle {
        TokenHandle(self.current)
    }

    /// Produce a [Span] starting at the position marked by [TokenHandle] and
    /// ending at the current location
    #[must_use]
    pub fn end_token(&mut self, TokenHandle(start): TokenHandle) -> Span {
        let current = self.current;
        Span {
            absolute: Some(AbsoluteSpan {
                start: start.loc,
                end: current.loc,
            }),
            relative: RelativeSpan {
                start: LineAndColumn {
                    line: start.line,
                    column: start.col,
                },
                end: LineAndColumn {
                    line: current.line,
                    column: current.col,
                },
            },
        }
    }

    /// Returns a wrapper iterator which can peek any number of items ahead
    /// before deciding whether to commit
    ///
    /// ```
    /// # use span::*;
    /// let mut chars = Chars::new("123456");
    ///
    /// // Checkpoint is an Iterator<Item = char>
    /// let mut checkpoint = chars.checkpoint();
    /// assert_eq!(checkpoint.next(), Some('1'));
    /// assert_eq!(checkpoint.next(), Some('2'));
    /// assert_eq!(checkpoint.next(), Some('3'));
    ///
    /// // After checkpoint has been iterated we can explicitly abort (or drop)
    /// // to leave the original iterator unmodified
    /// checkpoint.abort();
    /// assert_eq!(chars.next(), Some('1'));
    ///
    /// // Or we can commit to advance the original iterator to match what the
    /// // checkpoint yielded
    /// let mut checkpoint = chars.checkpoint();
    /// assert_eq!(checkpoint.next(), Some('2'));
    /// assert_eq!(checkpoint.next(), Some('3'));
    /// assert_eq!(checkpoint.next(), Some('4'));
    /// checkpoint.commit();
    /// assert_eq!(chars.next(), Some('5'));
    ///
    /// # // Internal check to confirm it works past the end of the iterator
    /// # let mut checkpoint = chars.checkpoint();
    /// # assert_eq!(checkpoint.next(), Some('6'));
    /// # assert_eq!(checkpoint.next(), None);
    /// # assert_eq!(checkpoint.next(), None);
    /// # checkpoint.commit();
    /// # assert_eq!(chars.next(), None);
    /// ```
    pub fn checkpoint(&mut self) -> Checkpoint<'_> {
        Checkpoint::new(self)
    }
}

impl Iterator for Chars {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.it.next()?;
        self.current.loc += 1;
        if next == '\n' {
            self.current.line += 1;
            self.current.col = 1;
        } else {
            self.current.col += 1;
        }
        Some(next)
    }
}

#[cfg_attr(coverage, coverage(off))]
impl PeekingNext for Chars {
    fn peeking_next<F>(&mut self, accept: F) -> Option<Self::Item>
    where
        Self: Sized,
        F: FnOnce(&Self::Item) -> bool,
    {
        let item = self.peek()?;
        if accept(&item) {
            let _ = self.next();
            Some(item)
        } else {
            None
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod test {
    use super::*;

    #[test]
    fn checkpoint_tracks_spans_correctly() {
        let mut chars = Chars::new("123456");
        let start = chars.start_token();
        let mut checkpoint = chars.checkpoint();
        assert_eq!(checkpoint.next(), Some('1'));
        assert_eq!(checkpoint.next(), Some('2'));
        assert_eq!(checkpoint.next(), Some('3'));
        checkpoint.commit();
        let span = chars.end_token(start);
        assert_eq!(format!("{span:#}"), "line 1 column 1 to column 4");
    }

    #[test]
    fn peek_while_tracks_spans_correctly() {
        let mut chars = Chars::new("111222");
        let start = chars.start_token();
        let _ = chars.peek_while(|c| c == '1').collect::<String>();
        let span = chars.end_token(start);
        assert_eq!(format!("{span:#}"), "line 1 column 1 to column 4");
    }
}
