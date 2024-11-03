use std::iter::Peekable;

use crate::{AbsoluteSpan, LineAndColumn, RelativeSpan, Span};

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
#[allow(missing_debug_implementations)]
pub struct Chars {
    it: Peekable<Box<dyn Iterator<Item = char>>>,
    current: Position,
}

impl Chars {
    /// New
    #[must_use]
    pub fn new(it: Box<dyn Iterator<Item = char>>) -> Self {
        Self {
            it: it.peekable(),
            current: Position {
                loc: 0,
                line: 1,
                col: 1,
            },
        }
    }

    /// Peek
    pub fn peek(&mut self) -> Option<char> {
        self.it.peek().copied()
    }

    /// Start Token
    #[must_use]
    pub fn start_token(&self) -> TokenHandle {
        TokenHandle(self.current)
    }

    /// End Token
    #[expect(clippy::needless_pass_by_value)]
    pub fn end_token(&mut self, handle: TokenHandle) -> Span {
        let start = handle.0;
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
