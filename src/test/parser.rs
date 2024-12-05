use crate::parser::{parse_tokens, Event, Operand, Operation};
use crate::token::Token::*;

#[test]
fn when_valid_tokens_expect_parsing_succeeds() {
    let input = vec![
        ThreadIdentifier(12),
        Pipe,
        Write,
        LeftParenthesis,
        MemoryLocation(6),
        RightParenthesis,
        Pipe,
        LineNumber(42),
    ];

    let expected_event = Event {
        thread_identifier: 12,
        operation: Operation::Write,
        operand: Operand::MemoryLocation(6),
        loc: 42,
    };

    let result = parse_tokens(input);
    assert!(result.is_ok());

    let trace = result.unwrap();
    assert_eq!(trace.events.len(), 1);
    assert_eq!(trace.events[0], expected_event);
}

#[test]
fn when_invalid_tokens_expect_parsing_fails() {
    let input = vec![
        ThreadIdentifier(12),
        Pipe,
        Write,
        Write, // repeated 'Write' token is invalid
        LeftParenthesis,
        MemoryLocation(6),
        LeftParenthesis,
        Write,
        LineNumber(42),
    ];

    let result = parse_tokens(input);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(error.to_string(), "error at 3: expected [LeftParenthesis]");
}
