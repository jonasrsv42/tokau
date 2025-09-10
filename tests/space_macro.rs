use tokau::{Name, Position, Space, Token, TokenSpace, range};

#[derive(Name, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
enum ControlToken {
    Start,
    Stop,
    Pause,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[range(1000)]
struct TextTokens(u32);

#[derive(Space, Debug, PartialEq, Clone, Copy)]
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
    assert_eq!(MySpace::position_of(ControlToken::Start), 0);
    assert_eq!(MySpace::position_of(ControlToken::Stop), 1);
    assert_eq!(MySpace::position_of(ControlToken::Pause), 2);

    // Test range token operations
    assert_eq!(MySpace::position_of(TextTokens(0)), 3);
    assert_eq!(MySpace::position_of(TextTokens(999)), 1002);
    // TextTokens(1000) would be out of bounds for the token itself

    // Test reverse lookups
    assert_eq!(
        MySpace::try_as::<ControlToken>(0),
        Some(ControlToken::Start)
    );
    assert_eq!(
        MySpace::try_as::<ControlToken>(2),
        Some(ControlToken::Pause)
    );
    assert_eq!(MySpace::try_as::<ControlToken>(3), None); // Text token range

    // Test range token lookups
    assert_eq!(MySpace::try_as::<TextTokens>(3), Some(TextTokens(0)));
    assert_eq!(MySpace::try_as::<TextTokens>(1002), Some(TextTokens(999)));
    assert_eq!(MySpace::try_as::<TextTokens>(1003), None);

    // Test dynamic tokens
    assert_eq!(MySpace::remainder(1002), None); // In static range
    assert_eq!(MySpace::remainder(1003), Some(0)); // First dynamic token
    assert_eq!(MySpace::remainder(2000), Some(997)); // Dynamic token at offset 997

    // Test decode method
    assert_eq!(
        MySpace::try_from(0).ok(),
        Some(MySpace::Control(ControlToken::Start))
    );
    assert_eq!(
        MySpace::try_from(1).ok(),
        Some(MySpace::Control(ControlToken::Stop))
    );
    assert_eq!(
        MySpace::try_from(2).ok(),
        Some(MySpace::Control(ControlToken::Pause))
    );
    assert_eq!(
        MySpace::try_from(3).ok(),
        Some(MySpace::Text(TextTokens(0)))
    );
    assert_eq!(
        MySpace::try_from(1002).ok(),
        Some(MySpace::Text(TextTokens(999)))
    );
    assert_eq!(MySpace::try_from(1003).ok(), Some(MySpace::Vocab(0)));
    assert_eq!(MySpace::try_from(2000).ok(), Some(MySpace::Vocab(997)));
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

#[test]
fn test_space_value_roundtrips() {
    // Test value() method for all space variants

    // Test ControlToken variants
    let control_start = MySpace::Control(ControlToken::Start);
    let control_stop = MySpace::Control(ControlToken::Stop);
    let control_pause = MySpace::Control(ControlToken::Pause);

    assert_eq!(control_start.value(), 0);
    assert_eq!(control_stop.value(), 1);
    assert_eq!(control_pause.value(), 2);

    // Test TextToken variants
    let text_first = MySpace::Text(TextTokens(0));
    let text_middle = MySpace::Text(TextTokens(500));
    let text_last = MySpace::Text(TextTokens(999));

    assert_eq!(text_first.value(), 3); // OFFSET (3) + 0
    assert_eq!(text_middle.value(), 503); // OFFSET (3) + 500
    assert_eq!(text_last.value(), 1002); // OFFSET (3) + 999

    // Test dynamic tokens
    let vocab_first = MySpace::Vocab(0);
    let vocab_middle = MySpace::Vocab(500);
    let vocab_high = MySpace::Vocab(1000);

    assert_eq!(vocab_first.value(), 1003); // RESERVED (1003) + 0
    assert_eq!(vocab_middle.value(), 1503); // RESERVED (1003) + 500
    assert_eq!(vocab_high.value(), 2003); // RESERVED (1003) + 1000

    // Test complete roundtrips: value -> try_from -> value
    let test_values = vec![
        // Control tokens
        0, 1, 2, // Text tokens
        3, 100, 500, 1002, // Dynamic tokens
        1003, 1500, 2000, 5000,
    ];

    for &original_value in &test_values {
        if let Ok(space) = MySpace::try_from(original_value) {
            let recovered_value = space.value();
            assert_eq!(
                recovered_value, original_value,
                "Roundtrip failed for value {}: got {}",
                original_value, recovered_value
            );
        } else {
            panic!("try_from failed for value {}", original_value);
        }
    }

    // Test reverse roundtrips: space -> value -> try_from -> compare
    let test_spaces = vec![
        MySpace::Control(ControlToken::Start),
        MySpace::Control(ControlToken::Stop),
        MySpace::Control(ControlToken::Pause),
        MySpace::Text(TextTokens(0)),
        MySpace::Text(TextTokens(42)),
        MySpace::Text(TextTokens(999)),
        MySpace::Vocab(0),
        MySpace::Vocab(100),
        MySpace::Vocab(1000),
    ];

    for original_space in test_spaces {
        let value = original_space.value();
        let recovered_space = MySpace::try_from(value).unwrap();
        assert_eq!(
            recovered_space, original_space,
            "Reverse roundtrip failed for space {:?}",
            original_space
        );
    }
}

#[test]
fn test_space_value_with_boundaries() {
    // Test boundary values specifically

    // Last control token
    let last_control = MySpace::Control(ControlToken::Pause);
    assert_eq!(last_control.value(), 2);
    assert_eq!(MySpace::try_from(2).unwrap(), last_control);

    // First text token
    let first_text = MySpace::Text(TextTokens(0));
    assert_eq!(first_text.value(), 3);
    assert_eq!(MySpace::try_from(3).unwrap(), first_text);

    // Last text token
    let last_text = MySpace::Text(TextTokens(999));
    assert_eq!(last_text.value(), 1002);
    assert_eq!(MySpace::try_from(1002).unwrap(), last_text);

    // First dynamic token
    let first_dynamic = MySpace::Vocab(0);
    assert_eq!(first_dynamic.value(), 1003);
    assert_eq!(MySpace::try_from(1003).unwrap(), first_dynamic);

    // Verify boundary transitions
    assert_ne!(MySpace::try_from(2).unwrap(), MySpace::try_from(3).unwrap()); // Control -> Text
    assert_ne!(
        MySpace::try_from(1002).unwrap(),
        MySpace::try_from(1003).unwrap()
    ); // Text -> Dynamic

    // Verify is_reserved matches our expectations
    assert!(MySpace::is_reserved(0)); // Control
    assert!(MySpace::is_reserved(2)); // Control
    assert!(MySpace::is_reserved(3)); // Text
    assert!(MySpace::is_reserved(1002)); // Text
    assert!(!MySpace::is_reserved(1003)); // Dynamic
    assert!(!MySpace::is_reserved(5000)); // Dynamic
}
