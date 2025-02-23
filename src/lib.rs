//! Helpers for handling spans
#![warn(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(noop_method_call)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_results)]
#![deny(missing_debug_implementations)]
#![deny(missing_copy_implementations)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::let_underscore_untyped)]
#![allow(clippy::similar_names)]
#![cfg_attr(coverage, feature(coverage_attribute))]

use std::fmt;

use serde::{Deserialize, Serialize};

pub use self::chars::{Chars, Checkpoint, TokenHandle};

mod chars;

/// Represents a region of a source file
///
/// # Examples
/// Empty span
/// ```
/// # use span::*;
/// let mut chars = Chars::new("123456");
/// let start = chars.start_token();
/// let span = chars.end_token(start);
/// assert_eq!(format!("{span}"), "line 1 column 1");
/// assert_eq!(format!("{span:#}"), "line 1 column 1");
/// ```
/// Single character span
/// ```
/// # use span::*;
/// let mut chars = Chars::new("123456");
/// let start = chars.start_token();
/// let _ = chars.next();
/// let span = chars.end_token(start);
/// assert_eq!(format!("{span}"), "line 1 column 1");
/// assert_eq!(format!("{span:#}"), "line 1 column 1");
/// ```
/// Single line span
/// ```
/// # use span::*;
/// let mut chars = &mut Chars::new("123456");
/// let start = chars.start_token();
/// for _ in chars.take(4) {}
/// let span = chars.end_token(start);
/// assert_eq!(format!("{span}"), "line 1 column 1");
/// assert_eq!(format!("{span:#}"), "line 1 column 1 to column 5");
/// ```
/// Multi line span
/// ```
/// # use span::*;
/// let mut chars = &mut Chars::new("123\n456");
/// let start = chars.start_token();
/// for _ in chars.take(5) {}
/// let span = chars.end_token(start);
/// assert_eq!(format!("{span}"), "line 1 column 1");
/// assert_eq!(format!("{span:#}"), "line 1 column 1 to line 2 column 2");
/// ```
/// Unknown span
/// ```
/// # use span::*;
/// assert_eq!(format!("{}", Span::UNKNOWN), "???");
/// assert_eq!(format!("{:#}", Span::UNKNOWN), "???");
/// ```
/// Unknown spans are considered equal to all other spans
/// ```
/// # use span::*;
/// let mut chars = &mut Chars::new("123456");
/// let span1 = {
///     let start = chars.start_token();
///     for _ in chars.take(3) {}
///     chars.end_token(start)
/// };
/// let span2 = {
///     let start = chars.start_token();
///     for _ in chars.take(3) {}
///     chars.end_token(start)
/// };
/// assert_eq!(span1, span1);
/// assert_ne!(span1, span2);
/// assert_eq!(span1, Span::UNKNOWN);
/// assert_eq!(span2, Span::UNKNOWN);
/// ```
#[derive(Debug, Copy, Clone)]
#[cfg_attr(not(coverage), derive(Serialize, Deserialize))]
pub struct Span {
    absolute: Option<AbsoluteSpan>,
    relative: RelativeSpan,
}

#[cfg_attr(coverage, coverage(off))]
impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_unknown() {
            write!(f, "???")?;
            return Ok(());
        }

        write!(
            f,
            "line {} column {}",
            self.relative.start.line, self.relative.start.column
        )?;

        // If the span is empty stop at printing the start character location
        if self.relative.start == self.relative.end {
            return Ok(());
        }

        // As above if the span is only 1 character wide
        if self.relative.start.line == self.relative.end.line
            && self.relative.start.column + 1 == self.relative.end.column
        {
            return Ok(());
        }

        // If # is specified and the span is more than 1 character wide print
        // the end
        if f.alternate() {
            write!(f, " to")?;
            #[allow(clippy::if_not_else)]
            if self.relative.start.line != self.relative.end.line {
                write!(f, " line {}", self.relative.end.line)?;
            }
            write!(f, " column {}", self.relative.end.column)?;
        }
        Ok(())
    }
}

impl Span {
    /// Placeholder for an unknown span
    pub const UNKNOWN: Span = Span {
        absolute: None,
        relative: RelativeSpan::UNKNOWN,
    };

    /// Take a list of spans and produce a span that covers all of them
    ///
    /// Aggregating an empty list of spans is an error. In debug it panics but
    /// in release it returns [Span::UNKNOWN]
    ///
    /// ```
    /// # use span::*;
    /// let mut chars = &mut Chars::new("123\n456\n789");
    /// let span1 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// assert_eq!(format!("{span1:#}"), "line 1 column 1 to column 4");
    /// assert_eq!(chars.next(), Some('\n'));
    /// let span2 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// assert_eq!(format!("{span2:#}"), "line 2 column 1 to column 4");
    /// assert_eq!(chars.next(), Some('\n'));
    /// let span3 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// assert_eq!(format!("{span3:#}"), "line 3 column 1 to column 4");
    /// assert_eq!(chars.next(), None);
    ///
    /// assert_eq!(
    ///     format!("{:#}", Span::aggregate(&[span1, span2, span3])),
    ///     "line 1 column 1 to line 3 column 4"
    /// );
    /// assert_eq!(
    ///     format!("{:#}", Span::aggregate(&[span1, span3])),
    ///     "line 1 column 1 to line 3 column 4"
    /// );
    /// assert_eq!(
    ///     format!("{:#}", Span::aggregate(&[span2, span3])),
    ///     "line 2 column 1 to line 3 column 4"
    /// );
    /// assert_eq!(
    ///     format!("{:#}", Span::aggregate(&[span1, span2])),
    ///     "line 1 column 1 to line 2 column 4"
    /// );
    /// ```
    /// # Panics
    /// If aggregating an empty list of spans in debug
    pub fn aggregate(spans: &[Span]) -> Span {
        #[cfg_attr(coverage, coverage(off))]
        fn check_unknown(span: &Span) {
            debug_assert!(
                !span.is_unknown(),
                "Attempted to aggregate an empty list of spans"
            );
        }
        let result = spans
            .iter()
            .copied()
            .reduce(Span::add)
            .unwrap_or(Span::UNKNOWN);
        check_unknown(&result);
        result
    }

    fn add(a: Span, b: Span) -> Span {
        if a.is_unknown() {
            return b;
        }
        if b.is_unknown() {
            return a;
        }
        Span {
            absolute: AbsoluteSpan::add(a.absolute, b.absolute),
            relative: RelativeSpan::add(a.relative, b.relative),
        }
    }

    /// Check if the span is Span::UNKNOWN, required as PartialEq is implemented
    /// such that Span:UNKNOWN is equal to all spans
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        self.absolute.is_none()
    }

    /// Start Line (1 indexed)
    ///
    /// ```
    /// # use span::*;
    /// let mut chars = &mut Chars::new("123\n456");
    /// let span1 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// assert_eq!(chars.next(), Some('\n'));
    /// let span2 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// # assert_eq!(chars.next(), None);
    /// assert_eq!(span1.start_line(), Some(1));
    /// assert_eq!(span2.start_line(), Some(2));
    /// assert_eq!(Span::UNKNOWN.start_line(), None);
    /// ```
    #[must_use]
    pub fn start_line(&self) -> Option<usize> {
        self.absolute.map(|_| self.relative.start.line)
    }

    /// Position on the start line of the beginning of the token (1 indexed)
    ///
    /// ```
    /// # use span::*;
    /// let mut chars = &mut Chars::new("123456");
    /// let span1 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// let span2 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// # assert_eq!(chars.next(), None);
    /// assert_eq!(span1.start_position_on_start_line(), Some(1));
    /// assert_eq!(span2.start_position_on_start_line(), Some(4));
    /// assert_eq!(Span::UNKNOWN.start_position_on_start_line(), None);
    /// ```
    #[must_use]
    pub fn start_position_on_start_line(&self) -> Option<usize> {
        self.absolute.map(|_| self.relative.start.column)
    }

    /// End Line (1 indexed)
    ///
    /// ```
    /// # use span::*;
    /// let mut chars = &mut Chars::new("123\n456");
    /// let span1 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// assert_eq!(chars.next(), Some('\n'));
    /// let span2 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// # assert_eq!(chars.next(), None);
    /// assert_eq!(span1.end_line(), Some(1));
    /// assert_eq!(span2.end_line(), Some(2));
    /// assert_eq!(Span::UNKNOWN.end_line(), None);
    /// ```
    #[must_use]
    pub fn end_line(&self) -> Option<usize> {
        self.absolute.map(|_| self.relative.end.line)
    }

    /// Position on the end line of the end of the token (1 indexed)
    ///
    /// ```
    /// # use span::*;
    /// let mut chars = &mut Chars::new("123456");
    /// let span1 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// let span2 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// # assert_eq!(chars.next(), None);
    /// assert_eq!(span1.end_position_on_end_line(), Some(4));
    /// assert_eq!(span2.end_position_on_end_line(), Some(7));
    /// assert_eq!(Span::UNKNOWN.end_position_on_end_line(), None);
    /// ```
    #[must_use]
    pub fn end_position_on_end_line(&self) -> Option<usize> {
        self.absolute.map(|_| self.relative.end.column)
    }

    /// Start of the token relative to the start of the text
    ///
    /// ```
    /// # use span::*;
    /// let mut chars = &mut Chars::new("123456");
    /// let span1 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// let span2 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// # assert_eq!(chars.next(), None);
    /// assert_eq!(span1.start(), Some(0));
    /// assert_eq!(span2.start(), Some(3));
    /// assert_eq!(Span::UNKNOWN.start(), None);
    /// ```
    #[must_use]
    pub fn start(&self) -> Option<usize> {
        Some(self.absolute?.start)
    }

    /// Length of the token (may span multiple lines)
    ///
    /// ```
    /// # use span::*;
    /// let mut chars = &mut Chars::new("123456");
    /// let span1 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// let span2 = {
    ///     let start = chars.start_token();
    ///     for _ in chars.take(3) {}
    ///     chars.end_token(start)
    /// };
    /// # assert_eq!(chars.next(), None);
    /// assert_eq!(span1.len(), Some(3));
    /// assert_eq!(span2.len(), Some(3));
    /// assert_eq!(Span::UNKNOWN.len(), None);
    /// ```
    #[must_use]
    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> Option<usize> {
        self.absolute.map(|s| s.end - s.start)
    }
}

#[cfg_attr(coverage, coverage(off))]
impl PartialEq for Span {
    fn eq(&self, other: &Span) -> bool {
        if self.is_unknown() || other.is_unknown() {
            return true;
        }
        self.absolute == other.absolute && self.relative == other.relative
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(not(coverage), derive(Serialize, Deserialize))]
struct AbsoluteSpan {
    start: usize,
    end: usize,
}

impl AbsoluteSpan {
    fn add(
        a: Option<AbsoluteSpan>,
        b: Option<AbsoluteSpan>,
    ) -> Option<AbsoluteSpan> {
        let a = a?;
        let b = b?;
        Some(AbsoluteSpan {
            start: usize::min(a.start, b.start),
            end: usize::max(a.end, b.end),
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(not(coverage), derive(Serialize, Deserialize))]
struct RelativeSpan {
    start: LineAndColumn,
    end: LineAndColumn,
}

impl RelativeSpan {
    const UNKNOWN: RelativeSpan = RelativeSpan {
        start: LineAndColumn::UNKNOWN,
        end: LineAndColumn::UNKNOWN,
    };

    fn add(a: RelativeSpan, b: RelativeSpan) -> RelativeSpan {
        RelativeSpan {
            start: LineAndColumn::min(a.start, b.start),
            end: LineAndColumn::max(a.end, b.end),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(not(coverage), derive(Serialize, Deserialize))]
struct LineAndColumn {
    line: usize,
    column: usize,
}

impl LineAndColumn {
    const UNKNOWN: LineAndColumn = LineAndColumn {
        line: usize::MAX,
        column: usize::MAX,
    };

    fn min(a: LineAndColumn, b: LineAndColumn) -> LineAndColumn {
        let (line, column) = (a.line, a.column).min((b.line, b.column));
        LineAndColumn { line, column }
    }

    fn max(a: LineAndColumn, b: LineAndColumn) -> LineAndColumn {
        let (line, column) = (a.line, a.column).max((b.line, b.column));
        LineAndColumn { line, column }
    }
}

#[cfg(test)]
#[cfg_attr(coverage, coverage(off))]
mod test {
    use rstest::rstest;

    use super::*;

    mod span {
        use pretty_assertions::assert_eq;

        use super::*;

        #[cfg(debug_assertions)]
        #[test]
        #[should_panic(
            expected = "Attempted to aggregate an empty list of spans"
        )]
        fn aggregate_empty_list() {
            let _ = Span::aggregate(&[]);
        }

        #[cfg(not(debug_assertions))]
        #[test]
        fn aggregate_empty_list() {
            assert_eq!(Span::aggregate(&[]), Span::UNKNOWN);
        }

        #[rstest]
        #[case(Span::UNKNOWN, Span::UNKNOWN, Span::UNKNOWN)]
        #[case(
            Span {
                absolute: Some(AbsoluteSpan { start: 1, end: 2 }),
                relative: RelativeSpan {
                    start: LineAndColumn { line: 4, column: 5 },
                    end: LineAndColumn { line: 6, column: 7 },
                },
            },
            Span {
                absolute: Some(AbsoluteSpan { start: 8, end: 9 }),
                relative: RelativeSpan {
                    start: LineAndColumn {
                        line: 10,
                        column: 11,
                    },
                    end: LineAndColumn {
                        line: 12,
                        column: 13,
                    },
                },
            },
            Span {
                absolute: Some(AbsoluteSpan { start: 1, end: 9 }),
                relative: RelativeSpan {
                    start: LineAndColumn {
                        line: 4,
                        column: 5,
                    },
                    end: LineAndColumn {
                        line: 12,
                        column: 13,
                    },
                },
            },
        )]
        #[case(
            Span {
                absolute: Some(AbsoluteSpan { start: 1, end: 2 }),
                relative: RelativeSpan {
                    start: LineAndColumn { line: 4, column: 5 },
                    end: LineAndColumn { line: 6, column: 7 },
                },
            },
            Span::UNKNOWN,
            Span {
                absolute: Some(AbsoluteSpan { start: 1, end: 2 }),
                relative: RelativeSpan {
                    start: LineAndColumn { line: 4, column: 5 },
                    end: LineAndColumn { line: 6, column: 7 },
                },
            },
        )]
        #[case(
            Span::UNKNOWN,
            Span {
                absolute: Some(AbsoluteSpan { start: 8, end: 9 }),
                relative: RelativeSpan {
                    start: LineAndColumn {
                        line: 10,
                        column: 11,
                    },
                    end: LineAndColumn {
                        line: 12,
                        column: 13,
                    },
                },
            },
            Span {
                absolute: Some(AbsoluteSpan { start: 8, end: 9 }),
                relative: RelativeSpan {
                    start: LineAndColumn {
                        line: 10,
                        column: 11,
                    },
                    end: LineAndColumn {
                        line: 12,
                        column: 13,
                    },
                },
            },
        )]
        fn add(
            #[case] left: Span,
            #[case] right: Span,
            #[case] expected: Span,
        ) {
            let actual = Span::add(left, right);
            assert_eq!(expected.is_unknown(), actual.is_unknown());
            assert_eq!(expected, actual);
        }

        #[rstest]
        #[case(Span::UNKNOWN, true)]
        #[case(
            Span {
                absolute: Some(AbsoluteSpan { start: 1, end: 2 }),
                relative: RelativeSpan {
                    start: LineAndColumn { line: 4, column: 5 },
                    end: LineAndColumn { line: 6, column: 7 },
                },
            },
            false,
        )]
        fn is_unknown(#[case] span: Span, #[case] expected: bool) {
            assert_eq!(span.is_unknown(), expected);
        }
    }

    mod absolute {
        use pretty_assertions::assert_eq;

        use super::*;

        #[rstest]
        #[case(None, None, None)]
        #[case(Some(AbsoluteSpan { start: 1, end: 2}), None, None)]
        #[case(None, Some(AbsoluteSpan { start: 3, end: 4}), None)]
        #[case(
            Some(AbsoluteSpan { start: 1, end: 2}),
            Some(AbsoluteSpan { start: 3, end: 4}),
            Some(AbsoluteSpan { start: 1, end: 4}),
        )]
        fn add(
            #[case] left: Option<AbsoluteSpan>,
            #[case] right: Option<AbsoluteSpan>,
            #[case] expected: Option<AbsoluteSpan>,
        ) {
            assert_eq!(expected, AbsoluteSpan::add(left, right));
        }
    }

    mod relative {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn add() {
            let left = RelativeSpan {
                start: LineAndColumn { line: 1, column: 2 },
                end: LineAndColumn { line: 3, column: 4 },
            };
            let right = RelativeSpan {
                start: LineAndColumn { line: 5, column: 6 },
                end: LineAndColumn { line: 7, column: 8 },
            };
            let expected = RelativeSpan {
                start: LineAndColumn { line: 1, column: 2 },
                end: LineAndColumn { line: 7, column: 8 },
            };
            assert_eq!(expected, RelativeSpan::add(left, right));
        }
    }

    mod line_and_column {
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn min() {
            let left = LineAndColumn { line: 1, column: 2 };
            let right = LineAndColumn { line: 3, column: 4 };
            assert_eq!(left, LineAndColumn::min(left, right));
        }

        #[test]
        fn max() {
            let left = LineAndColumn { line: 1, column: 2 };
            let right = LineAndColumn { line: 3, column: 4 };
            assert_eq!(right, LineAndColumn::max(left, right));
        }
    }
}
