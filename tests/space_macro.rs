use tokau::{Name, NameToken, Position, RangeToken, Space, Token, TokenSpace, range};

#[derive(Name, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
enum ControlToken {
    Start,
    Stop,
    Pause,
}

#[derive(Debug, PartialEq)]
#[range(1000)]
struct TextTokens(u32);

#[derive(Space, Debug, PartialEq)]
enum MySpace {
    Control(ControlToken),
    Text(TextTokens),
    #[dynamic]
    Vocab(u32),
}

#[test]
fn test_space_macro() {
    // Test that Position implementations were generated correctly
    assert_eq!(<MySpace as Position<ControlToken>>::OFFSET, 0);
    assert_eq!(
        <MySpace as Position<TextTokens>>::OFFSET,
        ControlToken::COUNT
    );

    // Test RESERVED calculation
    assert_eq!(MySpace::RESERVED, ControlToken::COUNT + TextTokens::COUNT);
    assert_eq!(MySpace::RESERVED, 3 + 1000);

    // Test static token operations
    assert_eq!(ControlToken::Start.inside::<MySpace>(), 0);
    assert_eq!(ControlToken::Stop.inside::<MySpace>(), 1);
    assert_eq!(ControlToken::Pause.inside::<MySpace>(), 2);

    // Test range token operations
    assert_eq!(TextTokens::inside::<MySpace>(0), Some(3));
    assert_eq!(TextTokens::inside::<MySpace>(999), Some(1002));
    assert_eq!(TextTokens::inside::<MySpace>(1000), None);

    // Test reverse lookups
    assert_eq!(MySpace::is::<ControlToken>(0), Some(ControlToken::Start));
    assert_eq!(MySpace::is::<ControlToken>(2), Some(ControlToken::Pause));
    assert_eq!(MySpace::is::<ControlToken>(3), None); // Text token range

    // Test range token lookups
    assert_eq!(MySpace::is::<TextTokens>(3), Some(TextTokens(0)));
    assert_eq!(MySpace::is::<TextTokens>(1002), Some(TextTokens(999)));
    assert_eq!(MySpace::is::<TextTokens>(1003), None);

    // Test dynamic tokens
    assert_eq!(MySpace::remainder(1002), None); // In static range
    assert_eq!(MySpace::remainder(1003), Some(0)); // First dynamic token
    assert_eq!(MySpace::remainder(2000), Some(997)); // Dynamic token at offset 997

    // Test decode method
    assert_eq!(
        MySpace::decode(0),
        Some(MySpace::Control(ControlToken::Start))
    );
    assert_eq!(
        MySpace::decode(1),
        Some(MySpace::Control(ControlToken::Stop))
    );
    assert_eq!(
        MySpace::decode(2),
        Some(MySpace::Control(ControlToken::Pause))
    );
    assert_eq!(MySpace::decode(3), Some(MySpace::Text(TextTokens(0))));
    assert_eq!(MySpace::decode(1002), Some(MySpace::Text(TextTokens(999))));
    assert_eq!(MySpace::decode(1003), Some(MySpace::Vocab(0)));
    assert_eq!(MySpace::decode(2000), Some(MySpace::Vocab(997)));
}

#[test]
fn test_space_macro_enum_usage() {
    // Test that we can create enum variants
    let control_token = MySpace::Control(ControlToken::Start);
    let text_token = MySpace::Text(TextTokens(42));
    let vocab_token = MySpace::Vocab(100);

    // Test pattern matching
    match control_token {
        MySpace::Control(ControlToken::Start) => {}
        _ => panic!("Expected Control(Start)"),
    }

    match text_token {
        MySpace::Text(TextTokens(pos)) => assert_eq!(pos, 42),
        _ => panic!("Expected Text token"),
    }

    match vocab_token {
        MySpace::Vocab(pos) => assert_eq!(pos, 100),
        _ => panic!("Expected Vocab token"),
    }
}
