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

use std::{cmp, fmt};

use serde::{Deserialize, Serialize};

/// Represents a region of a source file
///
/// # Examples
/// Empty span
/// ```
/// # use span::*;
/// let span = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 1,
///         column: 1
///     }
/// };
/// assert_eq!(format!("{span}"), String::from("line 1 column 1"));
/// assert_eq!(format!("{span:#}"), String::from("line 1 column 1"))
/// ```
/// Single character span
/// ```
/// # use span::*;
/// let span = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 1,
///         column: 2
///     }
/// };
/// assert_eq!(format!("{span}"), String::from("line 1 column 1"));
/// assert_eq!(format!("{span:#}"), String::from("line 1 column 1"))
/// ```
/// Single line span
/// ```
/// # use span::*;
/// let span = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 1,
///         column: 50
///     }
/// };
/// assert_eq!(format!("{span}"), String::from("line 1 column 1"));
/// assert_eq!(format!("{span:#}"), String::from("line 1 column 1 to column 50"))
/// ```
/// Multi line span
/// ```
/// # use span::*;
/// let span = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 2,
///         column: 50
///     }
/// };
/// assert_eq!(format!("{span}"), String::from("line 1 column 1"));
/// assert_eq!(format!("{span:#}"), String::from("line 1 column 1 to line 2 column 50"))
/// ```
/// Unknown span
/// ```
/// # use span::*;
/// assert_eq!(format!("{}", Span::UNKNOWN), String::from("???"));
/// assert_eq!(format!("{:#}", Span::UNKNOWN), String::from("???"))
/// ```
/// Unknown spans are considered equal to all other spans
/// ```
/// # use span::*;
/// let span1 = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 2,
///         column: 50
///     }
/// };
/// let span2 = Span {
///     start: LineAndColumn {
///         line: 1,
///         column: 1
///     },
///     end: LineAndColumn {
///         line: 1,
///         column: 1
///     }
/// };
/// assert_eq!(span1, span1);
/// assert_ne!(span1, span2);
/// assert_eq!(span1, Span::UNKNOWN);
/// assert_eq!(span2, Span::UNKNOWN);
/// ```
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Span {
    #[allow(missing_docs)]
    pub start: LineAndColumn,
    #[allow(missing_docs)]
    pub end: LineAndColumn,
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

        write!(f, "line {} column {}", self.start.line, self.start.column)?;

        // If the span is empty stop at printing the start character location
        if self.start == self.end {
            return Ok(());
        }

        // As above if the span is only 1 character wide
        if self.start.line == self.end.line && self.start.column + 1 == self.end.column {
            return Ok(());
        }

        // If # is specified and the span is more than 1 character wide print
        // the end
        if f.alternate() {
            write!(f, " to")?;
            #[allow(clippy::if_not_else)]
            if self.start.line != self.end.line {
                write!(f, " line {}", self.end.line)?;
            }
            write!(f, " column {}", self.end.column)?;
        }
        Ok(())
    }
}

impl Span {
    /// Placeholder for an unknown span
    pub const UNKNOWN: Span = Span {
        start: LineAndColumn {
            line: usize::MAX,
            column: usize::MAX,
        },
        end: LineAndColumn {
            line: usize::MAX,
            column: usize::MAX,
        },
    };

    /// Take a list of spans and produce a span that covers all of them
    ///
    /// Aggregating an empty list of spans is an error. In debug it panics but
    /// in release it returns [Span::UNKNOWN]
    ///
    /// # Examples
    /// ```
    /// # use span::*;
    /// let span1 = Span {
    ///     start: LineAndColumn {
    ///         line: 1,
    ///         column: 1
    ///     },
    ///     end: LineAndColumn {
    ///         line: 2,
    ///         column: 2
    ///     }
    /// };
    /// let span2 = Span {
    ///     start: LineAndColumn {
    ///         line: 3,
    ///         column: 3
    ///     },
    ///     end: LineAndColumn {
    ///         line: 4,
    ///         column: 4
    ///     }
    /// };
    /// let span3 = Span {
    ///     start: LineAndColumn {
    ///         line: 5,
    ///         column: 5
    ///     },
    ///     end: LineAndColumn {
    ///         line: 6,
    ///         column: 6
    ///     }
    /// };
    /// let expected = Span {
    ///     start: LineAndColumn {
    ///         line: 1,
    ///         column: 1
    ///     },
    ///     end: LineAndColumn {
    ///         line: 6,
    ///         column: 6
    ///     }
    /// };
    /// let result = Span::aggregate(vec![span1, span2, span3]);
    /// assert_eq!(result, expected);
    /// ```
    /// # Panics
    /// If aggregating an empty list of spans in debug
    pub fn aggregate(spans: Vec<Span>) -> Span {
        let result = spans.into_iter().reduce(Span::add);
        if cfg!(debug_assertions) {
            result.expect("Attempted to aggregate an empty list of spans")
        } else {
            result.unwrap_or(Span::UNKNOWN)
        }
    }

    fn add(a: Span, b: Span) -> Span {
        Span {
            start: LineAndColumn {
                line: cmp::min(a.start.line, b.start.line),
                column: cmp::min(a.start.column, b.start.column),
            },
            end: LineAndColumn {
                line: cmp::max(a.end.line, b.end.line),
                column: cmp::max(a.end.column, b.end.column),
            },
        }
    }

    /// Check if the span is Span::UNKNOWN, required as PartialEq is implemented
    /// such that Span:UNKNOWN is equal to all spans
    #[must_use]
    pub fn is_unknown(&self) -> bool {
        self.start.line == usize::MAX
            && self.start.column == usize::MAX
            && self.end.line == usize::MAX
            && self.end.column == usize::MAX
    }
}

impl PartialEq for Span {
    fn eq(&self, rhs: &Span) -> bool {
        if self.is_unknown() || rhs.is_unknown() {
            return true;
        }
        self.start.line == rhs.start.line
            && self.start.column == rhs.start.column
            && self.end.line == rhs.end.line
            && self.end.column == rhs.end.column
    }
}

/// Represents a specific character in a source file
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct LineAndColumn {
    #[allow(missing_docs)]
    pub line: usize,
    #[allow(missing_docs)]
    pub column: usize,
}

syntax_abuse::tests! {
    #[cfg(debug_assertions)]
    #[test]
    #[should_panic(expected = "Attempted to aggregate an empty list of spans")]
    fn aggregate_empty_list() {
        Span::aggregate(vec![]);
    }

    #[cfg(not(debug_assertions))]
    testcase! {
        aggregate_empty_list,
        Span::aggregate(vec![]),
        Span::UNKNOWN
    }

    testcase! {
        add,
        Span::add(
            Span { start: LineAndColumn { line: 0, column: 1 }, end: LineAndColumn { line: 2, column: 3 } },
            Span { start: LineAndColumn { line: 4, column: 5 }, end: LineAndColumn { line: 6, column: 7 } }
        ),
        Span { start: LineAndColumn { line: 0, column: 1 }, end: LineAndColumn { line: 6, column: 7 } }
    }

    testcase! {
        is_unknown_true,
        Span::UNKNOWN.is_unknown(),
        true
    }

    testcase! {
        is_unknown_false,
        Span { start: LineAndColumn { line: 0, column: 0 }, end: LineAndColumn { line: 0, column: 0 } }.is_unknown(),
        false
    }
}
