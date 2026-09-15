use miette::{Diagnostic, SourceSpan};
use thiserror::Error;
use lexaf::Token;

#[derive(Debug, Error, Diagnostic)]
pub enum ParsafError {
    #[error("token not allowed here")]
    #[diagnostic(
        code(parsaf::not_allowed),
        help("remove '{token}' or provide a valid expression before it")
    )]
    NotAllowedHere {
        token: Token,
        #[label("'{token}' cannot be placed at the start of a statement.")]
        span: SourceSpan,
    },

    #[error("unexpected token found")]
    #[diagnostic(
        code(parsaf::unexpected_token),
        help("remove '{token}' and add the correct one.")
    )]
    UnexpectedToken {
        token: Token,
        #[label("'{token}' is unexpected here.")]
        span: SourceSpan,
    },

    #[error("expected: '{expected}'")]
    #[diagnostic(
        code(parsaf::expected_found),
        help("try adding: '{expected}' here.")
    )]
    ExpectedFound {
        expected: String,
        found: Token,
        #[label("expected '{expected}' here")]
        span: SourceSpan,
    },

    #[error("unexpected EOF")]
    #[diagnostic(
        code(parsaf::unexpected_eof),
        help("you might be missing a closing brace '}}' or bracket ']'?")
    )]
    UnexpectedEof,
}
