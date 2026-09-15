use miette::{Diagnostic, SourceSpan};
use thiserror::Error;
use lexaf::Token;

#[derive(Debug, Error, Diagnostic)]
pub enum ParsafError {
    #[error("token not allowed here: '{token:?}'")]
    #[diagnostic(
        code(parsaf::not_allowed),
        help("this token cannot be placed at the start of a statement.")
    )]
    NotAllowedHere {
        token: Token,
        #[label("remove this token or provide a valid expression before it")]
        span: SourceSpan,
    },

    #[error("unexpected token found: '{token:?}'")]
    #[diagnostic(
        code(parsaf::unexpected_token),
        help("this token is unexpected over here.")
    )]
    UnexpectedToken {
        token: Token,
        #[label("remove this token && add the correct one.")]
        span: SourceSpan,
    },

    #[error("expected: '{expected}', found: '{found:?}'")]
    #[diagnostic(
        code(parsaf::expected_but_found),
        help("replace this token with: '{expected}'")
    )]
    ExpectedFound {
        expected: String,
        found: Token,
        #[label("expected: '{expected}' here")]
        span: SourceSpan,
    },

    #[error("unexpected EOF")]
    #[diagnostic(
        code(parsaf::unexpected_eof),
        help("you might be missing a closing brace '}}' or bracket ']'")
    )]
    UnexpectedEof,
}
