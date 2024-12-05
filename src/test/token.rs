use crate::token::tokenize_source;

#[test]
fn when_valid_characters_expect_lexing_succeeds() {
    let input = "T6|w(4294967298)|59";

    let result = tokenize_source(input.to_string());
    assert!(result.is_ok());

    let tokens = result.unwrap();
    assert_eq!(tokens.len(), 8);
}

#[test]
fn when_invalid_characters_expect_lexing_fails() {
    let input = "T6|w(4294967298)*|59"; // '*' is an invalid character

    let result = tokenize_source(input.to_string());
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert_eq!(
        error.to_string(),
        "Logos encountered an non-ascii character"
    );
}
