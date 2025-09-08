use tokau::{Name, Position, Space, Token, TokenSpace};

#[derive(Name, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
enum ControlToken {
    Start,
    Stop,
    Pause,
    Resume,
}

#[derive(Name, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
enum DataToken {
    Read,
    Write,
    Delete,
}

// Define a custom token space using the Space derive macro
#[derive(Space, Debug, PartialEq)]
enum MyTokenSpace {
    Control(ControlToken),
    Data(DataToken),
}

#[test]
fn test_token_space_with_derived_tokens() {
    // Test ControlToken positions
    assert_eq!(MyTokenSpace::position_of(ControlToken::Start), 0);
    assert_eq!(MyTokenSpace::position_of(ControlToken::Stop), 1);
    assert_eq!(MyTokenSpace::position_of(ControlToken::Pause), 2);
    assert_eq!(MyTokenSpace::position_of(ControlToken::Resume), 3);

    // Test DataToken positions (offset by ControlToken::COUNT = 4)
    assert_eq!(MyTokenSpace::position_of(DataToken::Read), 4);
    assert_eq!(MyTokenSpace::position_of(DataToken::Write), 5);
    assert_eq!(MyTokenSpace::position_of(DataToken::Delete), 6);

    // Test reverse lookups
    assert_eq!(
        MyTokenSpace::try_as::<ControlToken>(0),
        Some(ControlToken::Start)
    );
    assert_eq!(
        MyTokenSpace::try_as::<ControlToken>(3),
        Some(ControlToken::Resume)
    );
    assert_eq!(MyTokenSpace::try_as::<ControlToken>(4), None); // Out of ControlToken range

    assert_eq!(MyTokenSpace::try_as::<DataToken>(4), Some(DataToken::Read));
    assert_eq!(
        MyTokenSpace::try_as::<DataToken>(6),
        Some(DataToken::Delete)
    );
    assert_eq!(MyTokenSpace::try_as::<DataToken>(7), None); // Out of DataToken range

    // Test static values
    assert_eq!(MyTokenSpace::RESERVED, 7); // 4 + 3

    // Test decode method
    assert_eq!(
        MyTokenSpace::try_from(0).ok(),
        Some(MyTokenSpace::Control(ControlToken::Start))
    );
    assert_eq!(
        MyTokenSpace::try_from(3).ok(),
        Some(MyTokenSpace::Control(ControlToken::Resume))
    );
    assert_eq!(
        MyTokenSpace::try_from(4).ok(),
        Some(MyTokenSpace::Data(DataToken::Read))
    );
    assert_eq!(
        MyTokenSpace::try_from(6).ok(),
        Some(MyTokenSpace::Data(DataToken::Delete))
    );
    assert_eq!(MyTokenSpace::try_from(7).ok(), None); // Out of range
}

#[test]
fn test_dynamic_tokens_with_derived() {
    // Test dynamic token range (no bounds checking)
    assert_eq!(MyTokenSpace::remainder(0), None); // Control token
    assert_eq!(MyTokenSpace::remainder(6), None); // Data token
    assert_eq!(MyTokenSpace::remainder(7), Some(0)); // First dynamic token
    assert_eq!(MyTokenSpace::remainder(56), Some(49)); // Dynamic token at offset 49
}
