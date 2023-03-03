use std::num::ParseFloatError;

use std::num::ParseIntError;

use miette::Diagnostic;
use thiserror::Error;
use winnow;

use miette::SourceSpan;
use winnow::error::FromExternalError;

// use crate::formats::util::ParseErrorKind;
// use crate::formats::util::SyntaxParseError;

use super::mesh;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("oops!")]
    Syntax(winnow::error::ErrMode<winnow::error::Error<String>>),
    #[error("Missing sub-section {name}")]
    #[diagnostic(code(fds_tbx::smv::missing_sub_section))]
    MissingSubSection {
        #[label("In this section")]
        parent: SourceSpan,
        #[help("Expected a sub-section {name}")]
        name: &'static str,
        #[label("Found this instead")]
        found: Option<SourceSpan>,
    },
    #[error("Missing section {name}")]
    #[diagnostic(code(fds_tbx::smv::missing_section))]
    MissingSection {
        #[help("Expected a section {name}")]
        name: &'static str,
    },
    #[error("Reference to undefined {key_type}")]
    #[diagnostic(code(fds_tbx::smv::invalid_key))]
    InvalidKey {
        #[label("This key is invalid")]
        key: SourceSpan,
        #[help("Expected a reference to a {name}")]
        key_type: &'static str,
    },
}

impl<S: Into<String>> From<winnow::error::ErrMode<winnow::error::Error<S>>> for Error {
    fn from(err: winnow::error::ErrMode<winnow::error::Error<S>>) -> Self {
        Self::Syntax(err.map_input(|e| e.into()))
    }
}

#[derive(Debug, Error, Diagnostic)]
// #[error("oops!")]
// #[diagnostic(
//     // code(oops::my::bad),
//     // url(docsrs),
//     // help("try doing it better next time?")
// )]
pub enum ErrorOld {
    #[error("oops!")]
    WrongSyntax {
        #[label("here")]
        pos: SourceSpan,
        #[source]
        #[diagnostic_source]
        err: ErrorKind,
    },
    // #[error("oops!")]
    // Nom(#[source] winnow::error::ErrMode<winnow::error::Error<String>>),
    // TODO: Using enum instead of a &str worth it?
    #[error("oops!")]
    MissingSection { name: &'static str },
    #[error("oops!")]
    MissingSubSection {
        parent: SourceSpan,
        name: &'static str,
    },
    #[error("oops!")]
    InvalidKey { parent: SourceSpan, key: SourceSpan },
    // #[error(transparent)]
    // #[diagnostic(code(oops::my::bad))]
    // SyntaxError(#[from] SyntaxParseError<String>),
}

#[derive(Debug, Error, Diagnostic)]
pub enum ErrorKind {
    /// An error occurred while parsing an integer.
    #[error(transparent)]
    #[diagnostic(code(fds_tbx::smv::parse_int))]
    ParseIntError(ParseIntError),

    /// An error occurred while parsing a floating point number.
    #[error(transparent)]
    #[diagnostic(code(fds_tbx::smv::parse_float))]
    ParseFloatError(ParseFloatError),
    // // TODO: Expected is currently a lower bound, not an exact value because of the way the macro is written
    // WrongNumberOfValues { expected: usize, got: usize },
    // TrailingCharacters,
    // UnknownSection,
    // MismatchedIndex { expected: usize, got: usize },
    // Mesh(mesh::ErrorKind),
    // InvalidSection,
}

// impl<'a> FromExternalError<&'a str, ParseIntError> for SyntaxParseError<&'a str, ErrorKind> {
//     fn from_external_error(input: &'a str, _kind: winnow::error::ErrorKind, e: ParseIntError) -> Self {
//         SyntaxParseError {
//             input,
//             len: 0,
//             label: None,
//             help: None,
//             context: None,
//             kind: Some(ErrorKind::ParseIntError(e)),
//             touched: false,
//         }
//     }
// }

// impl<'a> FromExternalError<&'a str, ParseFloatError> for SyntaxParseError<&'a str, ErrorKind> {
//     fn from_external_error(input: &'a str, _kind: winnow::error::ErrorKind, e: ParseFloatError) -> Self {
//         SyntaxParseError {
//             input,
//             len: 0,
//             label: None,
//             help: None,
//             context: None,
//             kind: Some(ErrorKind::ParseFloatError(e)),
//             touched: false,
//         }
//     }
// }
