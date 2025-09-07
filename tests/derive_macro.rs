use tokau::{Name, NameToken, Token};

#[derive(Name, Debug, PartialEq, Clone, Copy)]
enum LanguageToken {
    If,
    Else,
    While,
    For,
    Function,
    Return,
}

#[derive(Name, Debug, PartialEq, Clone, Copy)]
enum OperatorToken {
    Plus,
    Minus,
    Multiply,
    Divide,
    Equals,
    NotEquals,
}

#[derive(Name, Debug, PartialEq, Clone, Copy)]
enum SimpleToken {
    Only,
}

#[test]
fn test_derive_macro_token_count() {
    assert_eq!(LanguageToken::COUNT, 6);
    assert_eq!(OperatorToken::COUNT, 6);
    assert_eq!(SimpleToken::COUNT, 1);
}

#[test]
fn test_derive_macro_values() {
    // Test LanguageToken values
    assert_eq!(LanguageToken::If.value(), 0);
    assert_eq!(LanguageToken::Else.value(), 1);
    assert_eq!(LanguageToken::While.value(), 2);
    assert_eq!(LanguageToken::For.value(), 3);
    assert_eq!(LanguageToken::Function.value(), 4);
    assert_eq!(LanguageToken::Return.value(), 5);

    // Test OperatorToken values
    assert_eq!(OperatorToken::Plus.value(), 0);
    assert_eq!(OperatorToken::Minus.value(), 1);
    assert_eq!(OperatorToken::Multiply.value(), 2);
    assert_eq!(OperatorToken::Divide.value(), 3);
    assert_eq!(OperatorToken::Equals.value(), 4);
    assert_eq!(OperatorToken::NotEquals.value(), 5);

    // Test SimpleToken
    assert_eq!(SimpleToken::Only.value(), 0);
}

#[test]
fn test_derive_macro_try_from() {
    // Test LanguageToken conversions
    assert_eq!(LanguageToken::try_from(0), Ok(LanguageToken::If));
    assert_eq!(LanguageToken::try_from(1), Ok(LanguageToken::Else));
    assert_eq!(LanguageToken::try_from(2), Ok(LanguageToken::While));
    assert_eq!(LanguageToken::try_from(3), Ok(LanguageToken::For));
    assert_eq!(LanguageToken::try_from(4), Ok(LanguageToken::Function));
    assert_eq!(LanguageToken::try_from(5), Ok(LanguageToken::Return));
    assert_eq!(LanguageToken::try_from(6), Err(())); // Out of range

    // Test OperatorToken conversions
    assert_eq!(OperatorToken::try_from(0), Ok(OperatorToken::Plus));
    assert_eq!(OperatorToken::try_from(5), Ok(OperatorToken::NotEquals));
    assert_eq!(OperatorToken::try_from(6), Err(())); // Out of range

    // Test SimpleToken conversions
    assert_eq!(SimpleToken::try_from(0), Ok(SimpleToken::Only));
    assert_eq!(SimpleToken::try_from(1), Err(())); // Out of range
}

#[test]
fn test_derive_macro_roundtrip() {
    // Test that value() and try_from() are inverses
    for i in 0..6 {
        let token = LanguageToken::try_from(i).unwrap();
        assert_eq!(token.value(), i);
    }

    for i in 0..6 {
        let token = OperatorToken::try_from(i).unwrap();
        assert_eq!(token.value(), i);
    }

    let simple = SimpleToken::try_from(0).unwrap();
    assert_eq!(simple.value(), 0);
}