use crate::error::{AnalyzerError, LexerError};
use crate::normalizer::normalize_tokens;
use logos::{Lexer, Logos};

#[derive(Logos, Debug, Copy, Clone)]
#[logos(skip r"[ \r\t\n\f]+")]
#[logos(error = LexerError)]
pub enum Token {
    // single char tokens
    #[token("|")]
    Pipe,
    #[token("(")]
    LeftParenthesis,
    #[token(")")]
    RightParenthesis,
    #[token("[")]
    LeftSquareBracket,
    #[token("]")]
    RightSquareBracket,
    #[token("w")]
    Write,
    #[token("r")]
    Read,
    // multi char tokens
    #[regex("T[0-9]+", id)]
    ThreadIdentifier(i64),
    #[regex("L[0-9]+", id)]
    LockIdentifier(i64),
    #[regex("V[0-9]+(\\.[0-9]+\\[[0-9]+\\])?", id)]
    MemoryLocation(i64),
    #[token("fork")]
    Fork,
    #[token("req")]
    Request,
    #[token("acq")]
    Acquire,
    #[token("rel")]
    Release,
    #[token("join")]
    Join,
    #[regex("[0-9]+", |lex| lex.slice().parse().ok())]
    LineNumber(i64),
}

pub fn tokenize_source(source: String, normalize: bool) -> Result<Vec<Token>, AnalyzerError> {
    let tokens = Token::lexer(&source)
        .collect::<Result<Vec<_>, LexerError>>()
        .map_err(AnalyzerError::from)?;

    if normalize {
        Ok(normalize_tokens(tokens))
    } else {
        Ok(tokens)
    }
}

fn id(lex: &mut Lexer<Token>) -> Option<i64> {
    let slice = lex.slice();
    let id = slice[1..slice.len()].parse().ok()?;

    Some(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn succeed_when_lexing_valid_chars() -> Result<(), AnalyzerError> {
        // arrange
        let input = read_to_string("test/valid_trace.std")?;

        // act
        let tokens = tokenize_source(input, false)?;

        // assert
        assert_eq!(tokens.len(), 8); // THREAD_IDENTIFIER PIPE WRITE LEFT_PAREN NUMBER RIGHT_PAREN PIPE NUMBER

        Ok(())
    }

    #[test]
    fn fail_when_lexing_invalid_chars() -> Result<(), AnalyzerError> {
        // arrange
        let input = read_to_string("test/unsupported_character.std")?;

        // act
        let error = tokenize_source(input.to_string(), false).unwrap_err();

        // assert
        assert!(match error {
            AnalyzerError::LexerError(LexerError::NonAsciiCharacter) => true,
            _ => false,
        });

        Ok(())
    }
}
