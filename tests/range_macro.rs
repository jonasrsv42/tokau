use tokau::{RangeToken, Token, TokenSpace, range};

#[derive(Debug, PartialEq)]
#[range(1000)]
struct TextTokens(u32);

#[derive(Debug, PartialEq)]
#[range(500)]
struct AudioTokens(u32);

#[test]
fn test_range_macro() {
    // Test that COUNT is correctly set
    assert_eq!(TextTokens::COUNT, 1000);
    assert_eq!(AudioTokens::COUNT, 500);

    // Test that we can create instances
    let text = TextTokens(42);
    assert_eq!(text.0, 42);

    let audio = AudioTokens(100);
    assert_eq!(audio.0, 100);
}

#[test]
fn test_range_token_inside() {
    use tokau::{Position, Space};

    // Define a simple space for testing using Space derive macro
    #[derive(Space, Debug, PartialEq)]
    enum TestSpace {
        Text(TextTokens),
        Audio(AudioTokens),
    }

    // Test RangeToken::inside
    assert_eq!(TextTokens::inside::<TestSpace>(0), Some(0));
    assert_eq!(TextTokens::inside::<TestSpace>(999), Some(999));
    assert_eq!(TextTokens::inside::<TestSpace>(1000), None); // Out of bounds

    assert_eq!(AudioTokens::inside::<TestSpace>(0), Some(1000));
    assert_eq!(AudioTokens::inside::<TestSpace>(499), Some(1499));
    assert_eq!(AudioTokens::inside::<TestSpace>(500), None); // Out of bounds
}
