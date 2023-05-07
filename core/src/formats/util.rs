use std::{fmt::Debug, str::FromStr};

use miette::SourceSpan;

use winnow::{
    ascii::space0,
    bytes::take_till1,
    sequence::preceded,
    stream::{AsChar, Stream, StreamIsPartial},
    IResult, Parser,
};

// Stolen from [kdl-rs](https://github.com/kdl-org/kdl-rs/blob/main/src/parser.rs)

/// Stores the entire input to work out where in the file errors occured.
///
/// All of our parsing subroutines want to hold onto some global information
/// to generate things like spans, so instead of making them simple free
/// functions, we wrap their bodies in closures that take in a [InputLocator](self::InputLocator).
/// The free functions then becoming constructors that return those closures.
/// This is basically the same idea behind winnow combinators like [many0](winnow::multi::many0) which
/// take an input to configure the combinator and then return a function.
#[derive(Debug)]
pub struct InputLocator<'a> {
    pub full_input: &'a str,
}

// /// A type reprenting additional information specific to the type of error being returned.
// #[derive(Debug, Diagnostic, Clone, Eq, PartialEq, Error)]
// pub enum ParseErrorKind {
//     /// An error occurred while parsing an integer.
//     #[error(transparent)]
//     #[diagnostic(code(kdl::parse_int))]
//     ParseIntError(ParseIntError),

//     /// An error occurred while parsing a floating point number.
//     #[error(transparent)]
//     #[diagnostic(code(kdl::parse_float))]
//     ParseFloatError(ParseFloatError),

// }

// #[derive(Debug, Clone, Eq, PartialEq)]
// pub(crate) struct SyntaxParseError<I, ErrorKind: Error> {
//     pub(crate) input: I,
//     pub(crate) context: Option<&'static str>,
//     pub(crate) len: usize,
//     pub(crate) label: Option<&'static str>,
//     pub(crate) help: Option<&'static str>,
//     pub(crate) kind: Option<ErrorKind>,
//     pub(crate) touched: bool,
// }

// /// A type reprenting additional information specific to the type of error being returned.
// #[derive(Debug, Diagnostic, Clone, Eq, PartialEq, Error)]
// enum GenericParseErrorKind<Kind: Error> {
//     #[error(transparent)]
//     Specific(#[from] Kind),

//     /// Generic parsing error. The given context string denotes the component
//     /// that failed to parse.
//     #[error("Expected {0}.")]
//     #[diagnostic(code(kdl::parse_component))]
//     Context(&'static str),

//     /// Generic unspecified error. If this is returned, the call site should
//     /// be annotated with context, if possible.
//     #[error("An unspecified error occurred.")]
//     #[diagnostic(code(kdl::other))]
//     Other,
// }

impl<'a> InputLocator<'a> {
    pub fn new(full_input: &'a str) -> Self {
        Self { full_input }
    }

    // pub fn parse<T, P, ErrKind: Error>(&self, parser: P) -> Result<T, ParseError<ErrKind>>
    // where
    //     P: Parser<&'a str, T, SyntaxParseError<&'a str, ErrKind>>,
    // {
    //     parser.parse_next(self.full_input)
    //         .finish()
    //         // .map(|(_, arg)| arg)
    //         .map_err(|e| {
    //             let span_substr = &e.input[..e.len];
    //             ParseError {
    //                 input: self.full_input.into(),
    //                 span: self.span_from_substr(span_substr),
    //                 help: e.help,
    //                 label: e.label,
    //                 kind: if let Some(kind) = e.kind {
    //                     GenericParseErrorKind::Specific(kind)
    //                 } else if let Some(ctx) = e.context {
    //                     GenericParseErrorKind::Context(ctx)
    //                 } else {
    //                     GenericParseErrorKind::Other
    //                 },
    //             }
    //         })
    // }

    /// Creates a span for an item using two substrings of self.full_input:
    ///
    /// * before: the remainder of the input before parsing the item
    /// * after: the remainder input after parsing the item
    ///
    /// All we really care about are the addresses of the strings, the lengths don't matter
    fn span_from_before_and_after(&self, before: &str, after: &str) -> SourceSpan {
        let base_addr = self.full_input.as_ptr() as usize;
        let before_addr = before.as_ptr() as usize;
        let after_addr = after.as_ptr() as usize;
        assert!(
            before_addr >= base_addr,
            "tried to get the span of a non-substring!"
        );
        assert!(
            after_addr >= before_addr,
            "subslices were in wrong order for spanning!"
        );

        let start = before_addr - base_addr;
        let end = after_addr - base_addr;
        SourceSpan::from(start..end)
    }

    /// Creates a span for an item using a substring of self.full_input
    ///
    /// Note that substr must be a literal substring, as in it must be
    /// a pointer into the same string!
    pub fn span_from_substr(&self, substr: &str) -> SourceSpan {
        let base_addr = self.full_input.as_ptr() as usize;
        let substr_addr = substr.as_ptr() as usize;
        assert!(
            substr_addr >= base_addr,
            "tried to get the span of a non-substring!"
        );
        let start = substr_addr - base_addr;
        let end = start + substr.len();
        SourceSpan::from(start..end)
    }
}

// impl<I, ErrKind: Error> winnow::error::ParseError<I> for SyntaxParseError<I, ErrKind> {
//     fn from_error_kind(input: I, _kind: ErrorKind) -> Self {
//         Self {
//             input,
//             len: 0,
//             label: None,
//             help: None,
//             context: None,
//             kind: None,
//             touched: false,
//         }
//     }

//     fn append(self, _input: I, _kind: ErrorKind) -> Self {
//         self
//     }
// }

// impl<I, ErrKind: Error> ContextError<I> for SyntaxParseError<I, ErrKind> {
//     fn add_context(self, _input: I, ctx: &'static str) -> Self {
//         self.context = self.context.or(Some(ctx));
//         self
//     }
// }

pub fn non_ws<I>(i: I) -> IResult<I, I::Slice>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar,
{
    take_till1(|x: <I as Stream>::Token| "\r\n\t ".contains(x.as_char()))
        .context("non_ws")
        .parse_next(i)
}

pub fn word<I>(i: I) -> IResult<I, I::Slice>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar + Copy,
{
    preceded(space0, non_ws).parse_next(i)
}

pub fn from_str<I, T: FromStr>(i: I, context: impl Debug + Clone) -> IResult<I, T>
where
    I: StreamIsPartial + Stream,
    <I as Stream>::Token: AsChar + Copy,
    <I as Stream>::Slice: AsRef<str>,
{
    non_ws
        .try_map(|x: I::Slice| x.as_ref().parse::<T>())
        .context(context)
        .parse_next(i)
}

// pub fn substr_to_span(full: &str, substr: &str) -> Range<usize> {
//     let offset = full.offset(substr);
//     offset..offset + substr.len()
// }

macro_rules! from_str_impl {
    ($($t:ident),+) => {
        $(pub fn $t<I>(i: I) -> IResult<I, $t>
        where
            I: StreamIsPartial + Stream,
            <I as Stream>::Token: AsChar + Copy,
            <I as Stream>::Slice: AsRef<str>,
        {
            from_str(i, stringify!($t))
        })+
    };
}

from_str_impl!(f32, i32, u32, usize);
