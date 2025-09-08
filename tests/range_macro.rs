use tokau::{RangeToken, Token, range};

#[range(1000)]
struct TextTokens(u32);

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
    use tokau::{Position, TokenSpace};

    // Define a simple space for testing
    struct TestSpace;

    impl Position<TextTokens> for TestSpace {
        const OFFSET: u32 = 0;
    }

    impl Position<AudioTokens> for TestSpace {
        const OFFSET: u32 = 1000;
    }

    impl TokenSpace for TestSpace {
        const RESERVED: u32 = TextTokens::COUNT + AudioTokens::COUNT;

        fn count(&self) -> u32 {
            Self::RESERVED
        }
    }

    // Test RangeToken::inside
    assert_eq!(TextTokens::inside::<TestSpace>(0), Some(0));
    assert_eq!(TextTokens::inside::<TestSpace>(999), Some(999));
    assert_eq!(TextTokens::inside::<TestSpace>(1000), None); // Out of bounds

    assert_eq!(AudioTokens::inside::<TestSpace>(0), Some(1000));
    assert_eq!(AudioTokens::inside::<TestSpace>(499), Some(1499));
    assert_eq!(AudioTokens::inside::<TestSpace>(500), None); // Out of bounds
}
