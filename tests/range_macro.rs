use tokau::{Token, TokenSpace, range};

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

    // Test range token position calculation
    assert_eq!(TestSpace::position_of(TextTokens(0)), 0);
    assert_eq!(TestSpace::position_of(TextTokens(999)), 999);
    // TextTokens(1000) would be out of bounds for the token itself

    assert_eq!(TestSpace::position_of(AudioTokens(0)), 1000);
    assert_eq!(TestSpace::position_of(AudioTokens(499)), 1499);
    // AudioTokens(500) would be out of bounds for the token itself
}
