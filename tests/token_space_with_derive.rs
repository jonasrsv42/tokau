use tokau::{Name, NameToken, Position, Token, TokenSpace};

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

// Define a custom token space
struct MyTokenSpace {
    dynamic_count: u32,
}

impl Position<ControlToken> for MyTokenSpace {
    const OFFSET: u32 = 0;
}

impl Position<DataToken> for MyTokenSpace {
    const OFFSET: u32 = ControlToken::COUNT;
}

impl TokenSpace for MyTokenSpace {
    const RESERVED: u32 = ControlToken::COUNT + DataToken::COUNT;

    fn count(&self) -> u32 {
        Self::RESERVED + self.dynamic_count
    }
}

#[test]
fn test_token_space_with_derived_tokens() {
    let space = MyTokenSpace { dynamic_count: 100 };

    // Test ControlToken positions
    assert_eq!(ControlToken::Start.inside::<MyTokenSpace>(), 0);
    assert_eq!(ControlToken::Stop.inside::<MyTokenSpace>(), 1);
    assert_eq!(ControlToken::Pause.inside::<MyTokenSpace>(), 2);
    assert_eq!(ControlToken::Resume.inside::<MyTokenSpace>(), 3);

    // Test DataToken positions (offset by ControlToken::COUNT = 4)
    assert_eq!(DataToken::Read.inside::<MyTokenSpace>(), 4);
    assert_eq!(DataToken::Write.inside::<MyTokenSpace>(), 5);
    assert_eq!(DataToken::Delete.inside::<MyTokenSpace>(), 6);

    // Test reverse lookups
    assert_eq!(
        MyTokenSpace::is::<ControlToken>(0),
        Some(ControlToken::Start)
    );
    assert_eq!(
        MyTokenSpace::is::<ControlToken>(3),
        Some(ControlToken::Resume)
    );
    assert_eq!(MyTokenSpace::is::<ControlToken>(4), None); // Out of ControlToken range

    assert_eq!(MyTokenSpace::is::<DataToken>(4), Some(DataToken::Read));
    assert_eq!(MyTokenSpace::is::<DataToken>(6), Some(DataToken::Delete));
    assert_eq!(MyTokenSpace::is::<DataToken>(7), None); // Out of DataToken range

    // Test total space count
    assert_eq!(space.count(), 107); // 4 + 3 + 100
    assert_eq!(MyTokenSpace::RESERVED, 7); // 4 + 3
}

#[test]
fn test_dynamic_tokens_with_derived() {
    let space = MyTokenSpace { dynamic_count: 50 };

    // Test dynamic token range
    assert_eq!(space.dynamic(0), None); // Control token
    assert_eq!(space.dynamic(6), None); // Data token
    assert_eq!(space.dynamic(7), Some(0)); // First dynamic token
    assert_eq!(space.dynamic(56), Some(49)); // Last dynamic token
    assert_eq!(space.dynamic(57), None); // Out of range
}
