use crate::token::Token;

pub(crate) fn normalize_tokens(tokens: Vec<Token>) -> Vec<Token> {
    let mut normalized_tokens = Vec::new();
    let mut token_iterator = tokens.iter();

    while let Some(current_token) = token_iterator.next() {
        match current_token {
            Token::Write => {
                normalized_tokens.push(Token::Write);
                normalized_tokens.push(Token::LeftParenthesis);

                // skip 2 tokens
                token_iterator.next();
                let token = token_iterator.next().unwrap();

                if let Token::LineNumber(loc) = token {
                    normalized_tokens.push(Token::MemoryLocation(loc.to_owned()));
                }
            }
            Token::Read => {
                normalized_tokens.push(Token::Read);
                normalized_tokens.push(Token::LeftParenthesis);

                // skip 2 tokens
                token_iterator.next();
                let token = token_iterator.next().unwrap();

                if let Token::LineNumber(loc) = token {
                    normalized_tokens.push(Token::MemoryLocation(loc.to_owned()));
                }
            }
            Token::Fork => {
                normalized_tokens.push(Token::Fork);
                normalized_tokens.push(Token::LeftParenthesis);

                // skip 2 tokens
                token_iterator.next();
                let token = token_iterator.next().unwrap();

                if let Token::LineNumber(loc) = token {
                    normalized_tokens.push(Token::ThreadIdentifier(loc.to_owned()));
                }
            }
            Token::Join => {
                normalized_tokens.push(Token::Join);
                normalized_tokens.push(Token::LeftParenthesis);

                // skip 2 tokens
                token_iterator.next();
                let token = token_iterator.next().unwrap();

                if let Token::LineNumber(loc) = token {
                    normalized_tokens.push(Token::ThreadIdentifier(loc.to_owned()));
                }
            }
            Token::Request => {
                normalized_tokens.push(Token::Request);
                normalized_tokens.push(Token::LeftParenthesis);

                // skip 2 tokens
                token_iterator.next();
                let token = token_iterator.next().unwrap();

                if let Token::LineNumber(loc) = token {
                    normalized_tokens.push(Token::LockIdentifier(loc.to_owned()));
                }
            }
            Token::Acquire => {
                normalized_tokens.push(Token::Acquire);
                normalized_tokens.push(Token::LeftParenthesis);

                // skip 2 tokens
                token_iterator.next();
                let token = token_iterator.next().unwrap();

                if let Token::LineNumber(loc) = token {
                    normalized_tokens.push(Token::LockIdentifier(loc.to_owned()));
                }
            }
            Token::Release => {
                normalized_tokens.push(Token::Release);
                normalized_tokens.push(Token::LeftParenthesis);

                // skip 2 tokens
                token_iterator.next();
                let token = token_iterator.next().unwrap();

                if let Token::LineNumber(loc) = token {
                    normalized_tokens.push(Token::LockIdentifier(loc.to_owned()));
                }
            }
            _ => normalized_tokens.push(*current_token),
        }
    }

    normalized_tokens
}