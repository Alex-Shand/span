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

use std::fmt;

pub use chars::{Chars, TokenHandle};
use serde::{Deserialize, Serialize};

mod chars;

/// Represents a region of a source file
///
/// # Examples
/// Empty span
/// ```rust,ignore
/// # use span::*;
/// let span = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     abs_start: 1,
///     abs_end: 1,
/// };
/// assert_eq!(format!("{span}"), String::from("line 1 column 1"));
/// assert_eq!(format!("{span:#}"), String::from("line 1 column 1"))
/// ```
/// Single character span
/// ```rust,ignore
/// # use span::*;
/// let span = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 1,
///         column: 2
///     },
///     abs_start: 1,
///     abs_end: 2
/// };
/// assert_eq!(format!("{span}"), String::from("line 1 column 1"));
/// assert_eq!(format!("{span:#}"), String::from("line 1 column 1"))
/// ```
/// Single line span
/// ```rust,ignore
/// # use span::*;
/// let span = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 1,
///         column: 50
///     },
///     abs_start: 1,
///     abs_end: 50,
/// };
/// assert_eq!(format!("{span}"), String::from("line 1 column 1"));
/// assert_eq!(format!("{span:#}"), String::from("line 1 column 1 to column 50"))
/// ```
/// Multi line span
/// ```rust,ignore
/// # use span::*;
/// let span = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 2,
///         column: 50
///     },
///     abs_start: 1,
///     abs_end: 100,
/// };
/// assert_eq!(format!("{span}"), String::from("line 1 column 1"));
/// assert_eq!(format!("{span:#}"), String::from("line 1 column 1 to line 2 column 50"))
/// ```
/// Unknown span
/// ```rust,ignore
/// # use span::*;
/// assert_eq!(format!("{}", Span::UNKNOWN), String::from("???"));
/// assert_eq!(format!("{:#}", Span::UNKNOWN), String::from("???"))
/// ```
/// Unknown spans are considered equal to all other spans
/// ```rust,ignore
/// # use span::*;
/// let span1 = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 2,
///         column: 50
///     },
///     abs_start: 1,
///     abs_end: 100,
/// };
/// let span2 = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     abs_start: 1,
///     abs_end: 1,
/// };
/// assert_eq!(span1, span1);
/// assert_ne!(span1, span2);
/// assert_eq!(span1, Span::UNKNOWN);
/// assert_eq!(span2, Span::UNKNOWN);
/// ```
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Span {
    absolute: Option<AbsoluteSpan>,
    relative: RelativeSpan,
}

impl fmt::Debug for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Span: {self:#}")
    }
}

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
    /// # Examples
    /// ```rust,ignore
    /// # use span::*;
    /// let span1 = Span {
    ///     start: LineAndColumn {
    ///         line: 1,
    ///         column: 1,
    ///     },
    ///     end: LineAndColumn {
    ///         line: 2,
    ///         column: 2
    ///     },
    ///     abs_start: 1,
    ///     abs_end: 2,
    /// };
    /// let span2 = Span {
    ///     start: LineAndColumn {
    ///         line: 3,
    ///         column: 3
    ///     },
    ///     end: LineAndColumn {
    ///         line: 4,
    ///         column: 4
    ///     },
    ///     abs_start: 3,
    ///     abs_end: 4
    /// };
    /// let span3 = Span {
    ///     start: LineAndColumn {
    ///         line: 5,
    ///         column: 5
    ///     },
    ///     end: LineAndColumn {
    ///         line: 6,
    ///         column: 6
    ///     },
    ///     abs_start: 5,
    ///     abs_end: 6,
    /// };
    /// let expected = Span {
    ///     start: LineAndColumn {
    ///         line: 1,
    ///         column: 1
    ///     },
    ///     end: LineAndColumn {
    ///         line: 6,
    ///         column: 6
    ///     },
    ///     abs_start: 1,
    ///     abs_end: 6,
    /// };
    /// let result = Span::aggregate(&[span1, span2, span3]);
    /// assert_eq!(result, expected);
    /// ```
    /// # Panics
    /// If aggregating an empty list of spans in debug
    pub fn aggregate(spans: &[Span]) -> Span {
        let result = spans.iter().copied().reduce(Span::add);
        if cfg!(debug_assertions) {
            result.expect("Attempted to aggregate an empty list of spans")
        } else {
            result.unwrap_or(Span::UNKNOWN)
        }
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

    /// Start Line
    #[must_use]
    pub fn start_line(&self) -> Option<usize> {
        self.absolute.map(|_| self.relative.start.line)
    }

    /// Position on the start line of the beginning of the token
    #[must_use]
    pub fn start_position_on_start_line(&self) -> Option<usize> {
        self.absolute.map(|_| self.relative.start.column)
    }

    /// End Line
    #[must_use]
    pub fn end_line(&self) -> Option<usize> {
        self.absolute.map(|_| self.relative.end.line)
    }

    /// Position on the end line of the end of the token
    #[must_use]
    pub fn end_position_on_end_line(&self) -> Option<usize> {
        self.absolute.map(|_| self.relative.end.column)
    }

    /// Start of the token relative to the start of the text
    #[must_use]
    pub fn start(&self) -> Option<usize> {
        Some(self.absolute?.start)
    }

    /// Length of the token (may span multiple lines)
    #[must_use]
    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> Option<usize> {
        self.absolute.map(|s| s.end - s.start)
    }
}

impl PartialEq for Span {
    fn eq(&self, other: &Span) -> bool {
        if self.is_unknown() || other.is_unknown() {
            return true;
        }
        self.absolute == other.absolute && self.relative == other.relative
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
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

impl PartialEq for AbsoluteSpan {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
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

impl PartialEq for RelativeSpan {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start && self.end == other.end
    }
}

/// Represents a specific character in a source file
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
struct LineAndColumn {
    #[allow(missing_docs)]
    pub line: usize,
    #[allow(missing_docs)]
    pub column: usize,
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

// #[cfg(test)]
// mod tests {
//     use pretty_assertions::assert_eq;
//     use rstest::rstest;

//     use super::*;

//     #[cfg(debug_assertions)]
//     #[test]
//     #[should_panic(expected = "Attempted to aggregate an empty list of spans")]
//     fn aggregate_empty_list() {
//         let _ = Span::aggregate(&[]);
//     }

//     #[cfg(not(debug_assertions))]
//     #[test]
//     fn aggregate_empty_list() {
//         assert_eq!(Span::aggregate(&[]), Span::UNKNOWN);
//     }

//     #[test]
//     fn add() {
//         assert_eq!(
//             Span::add(
//                 Span {
//                     start: LineAndColumn { line: 0, column: 1 },
//                     end: LineAndColumn { line: 2, column: 3 },
//                     abs_start: 8,
//                     abs_end: 9,
//                 },
//                 Span {
//                     start: LineAndColumn { line: 4, column: 5 },
//                     end: LineAndColumn { line: 6, column: 7 },
//                     abs_start: 10,
//                     abs_end: 11,
//                 }
//             ),
//             Span {
//                 start: LineAndColumn { line: 0, column: 1 },
//                 end: LineAndColumn { line: 6, column: 7 },
//                 abs_start: 8,
//                 abs_end: 11,
//             }
//         );
//     }

//     #[rstest]
//     #[case(Span::UNKNOWN, true)]
//     #[case(Span { start: LineAndColumn { line: 0, column: 0 }, end: LineAndColumn { line: 0, column: 0 }, abs_start: 0, abs_end: 0 }, false)]
//     fn is_unknown(#[case] span: Span, #[case] expected: bool) {
//         assert_eq!(span.is_unknown(), expected);
//     }
// }
