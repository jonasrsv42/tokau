use tokau::{Name, Position, Space, Token, TokenSpace, range};

// Test boundary conditions with small token counts
#[derive(Name, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
enum TinyToken {
    Only, // COUNT = 1
}

// Test with minimal range tokens
#[derive(Debug, PartialEq)]
#[range(1)]
struct SingleRangeToken(u32);

#[derive(Debug, PartialEq)]
#[range(0)]
struct ZeroRangeToken(u32);

// Complex layout: alternating Name and Range tokens
#[derive(Name, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
enum FirstName {
    A,
    B,
}

#[derive(Debug, PartialEq)]
#[range(3)]
struct FirstRange(u32);

#[derive(Name, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
enum SecondName {
    X,
    Y,
    Z,
}

#[derive(Debug, PartialEq)]
#[range(2)]
struct SecondRange(u32);

// Test spaces with different layouts
#[derive(Space, Debug, PartialEq)]
enum MinimalSpace {
    Tiny(TinyToken),
}

#[derive(Space, Debug, PartialEq)]
enum BoundarySpace {
    Single(SingleRangeToken),
    Tiny(TinyToken),
}

#[derive(Space, Debug, PartialEq)]
enum ComplexAlternatingSpace {
    FirstName(FirstName),
    FirstRange(FirstRange),
    SecondName(SecondName),
    SecondRange(SecondRange),
    #[dynamic]
    Dynamic(u32),
}

#[derive(Space, Debug, PartialEq)]
enum OnlyDynamicSpace {
    #[dynamic]
    Dynamic(u32),
}

#[test]
fn test_minimal_space_boundaries() {
    // MinimalSpace: TinyToken at 0..1
    assert_eq!(<MinimalSpace as Position<TinyToken>>::OFFSET, 0);
    assert_eq!(MinimalSpace::RESERVED, 1);

    // Test boundary conditions
    assert_eq!(MinimalSpace::try_as::<TinyToken>(0), Some(TinyToken::Only));
    assert_eq!(MinimalSpace::try_as::<TinyToken>(1), None); // Just outside

    assert_eq!(
        MinimalSpace::try_from(0).ok(),
        Some(MinimalSpace::Tiny(TinyToken::Only))
    );
    assert_eq!(MinimalSpace::try_from(1).ok(), None);

    // Test round trip
    assert_eq!(MinimalSpace::position_of(TinyToken::Only), 0);
}

#[test]
fn test_boundary_space_edge_cases() {
    // BoundarySpace layout:
    // - SingleRangeToken: 0..1 (1 token)
    // - TinyToken: 1..2 (1 token)

    assert_eq!(<BoundarySpace as Position<SingleRangeToken>>::OFFSET, 0);
    assert_eq!(<BoundarySpace as Position<TinyToken>>::OFFSET, 1);
    assert_eq!(BoundarySpace::RESERVED, 2);

    // Test SingleRangeToken boundaries
    assert_eq!(
        BoundarySpace::try_as::<SingleRangeToken>(0),
        Some(SingleRangeToken(0))
    );
    assert_eq!(BoundarySpace::try_as::<SingleRangeToken>(1), None); // Now in TinyToken range

    // Test TinyToken boundaries
    assert_eq!(BoundarySpace::try_as::<TinyToken>(1), Some(TinyToken::Only));
    assert_eq!(BoundarySpace::try_as::<TinyToken>(2), None); // Out of space
    assert_eq!(BoundarySpace::try_as::<TinyToken>(0), None); // In SingleRangeToken range

    // Test decode at boundaries
    assert_eq!(
        BoundarySpace::try_from(0).ok(),
        Some(BoundarySpace::Single(SingleRangeToken(0)))
    );
    assert_eq!(
        BoundarySpace::try_from(1).ok(),
        Some(BoundarySpace::Tiny(TinyToken::Only))
    );
    assert_eq!(BoundarySpace::try_from(2).ok(), None);

    // Test round trips
    assert_eq!(BoundarySpace::position_of(SingleRangeToken(0)), 0);
    // SingleRangeToken(1) would be out of bounds for the token itself
    assert_eq!(BoundarySpace::position_of(TinyToken::Only), 1);
}

#[test]
fn test_complex_alternating_layout() {
    // ComplexAlternatingSpace layout:
    // - FirstName: 0..2 (2 tokens)
    // - FirstRange: 2..5 (3 tokens)
    // - SecondName: 5..8 (3 tokens)
    // - SecondRange: 8..10 (2 tokens)
    // - Dynamic: 10+

    assert_eq!(<ComplexAlternatingSpace as Position<FirstName>>::OFFSET, 0);
    assert_eq!(<ComplexAlternatingSpace as Position<FirstRange>>::OFFSET, 2);
    assert_eq!(<ComplexAlternatingSpace as Position<SecondName>>::OFFSET, 5);
    assert_eq!(
        <ComplexAlternatingSpace as Position<SecondRange>>::OFFSET,
        8
    );
    assert_eq!(ComplexAlternatingSpace::RESERVED, 10);

    // Test each section's boundaries

    // FirstName: 0..2
    assert_eq!(
        ComplexAlternatingSpace::try_as::<FirstName>(0),
        Some(FirstName::A)
    );
    assert_eq!(
        ComplexAlternatingSpace::try_as::<FirstName>(1),
        Some(FirstName::B)
    );
    assert_eq!(ComplexAlternatingSpace::try_as::<FirstName>(2), None);

    // FirstRange: 2..5
    assert_eq!(
        ComplexAlternatingSpace::try_as::<FirstRange>(2),
        Some(FirstRange(0))
    );
    assert_eq!(
        ComplexAlternatingSpace::try_as::<FirstRange>(3),
        Some(FirstRange(1))
    );
    assert_eq!(
        ComplexAlternatingSpace::try_as::<FirstRange>(4),
        Some(FirstRange(2))
    );
    assert_eq!(ComplexAlternatingSpace::try_as::<FirstRange>(5), None);
    assert_eq!(ComplexAlternatingSpace::try_as::<FirstRange>(1), None); // In FirstName range

    // SecondName: 5..8
    assert_eq!(
        ComplexAlternatingSpace::try_as::<SecondName>(5),
        Some(SecondName::X)
    );
    assert_eq!(
        ComplexAlternatingSpace::try_as::<SecondName>(6),
        Some(SecondName::Y)
    );
    assert_eq!(
        ComplexAlternatingSpace::try_as::<SecondName>(7),
        Some(SecondName::Z)
    );
    assert_eq!(ComplexAlternatingSpace::try_as::<SecondName>(8), None);
    assert_eq!(ComplexAlternatingSpace::try_as::<SecondName>(4), None); // In FirstRange range

    // SecondRange: 8..10
    assert_eq!(
        ComplexAlternatingSpace::try_as::<SecondRange>(8),
        Some(SecondRange(0))
    );
    assert_eq!(
        ComplexAlternatingSpace::try_as::<SecondRange>(9),
        Some(SecondRange(1))
    );
    assert_eq!(ComplexAlternatingSpace::try_as::<SecondRange>(10), None); // Now dynamic
    assert_eq!(ComplexAlternatingSpace::try_as::<SecondRange>(7), None); // In SecondName range

    // Dynamic: 10+
    assert_eq!(ComplexAlternatingSpace::remainder(9), None); // Still in static range
    assert_eq!(ComplexAlternatingSpace::remainder(10), Some(0));
    assert_eq!(ComplexAlternatingSpace::remainder(100), Some(90));

    // Test decode across all ranges
    assert_eq!(
        ComplexAlternatingSpace::try_from(0).ok(),
        Some(ComplexAlternatingSpace::FirstName(FirstName::A))
    );
    assert_eq!(
        ComplexAlternatingSpace::try_from(2).ok(),
        Some(ComplexAlternatingSpace::FirstRange(FirstRange(0)))
    );
    assert_eq!(
        ComplexAlternatingSpace::try_from(5).ok(),
        Some(ComplexAlternatingSpace::SecondName(SecondName::X))
    );
    assert_eq!(
        ComplexAlternatingSpace::try_from(8).ok(),
        Some(ComplexAlternatingSpace::SecondRange(SecondRange(0)))
    );
    assert_eq!(
        ComplexAlternatingSpace::try_from(10).ok(),
        Some(ComplexAlternatingSpace::Dynamic(0))
    );
    assert_eq!(
        ComplexAlternatingSpace::try_from(100).ok(),
        Some(ComplexAlternatingSpace::Dynamic(90))
    );
}

#[test]
fn test_only_dynamic_space() {
    // OnlyDynamicSpace has RESERVED = 0, everything try_as dynamic
    assert_eq!(OnlyDynamicSpace::RESERVED, 0);

    assert_eq!(OnlyDynamicSpace::remainder(0), Some(0));
    assert_eq!(OnlyDynamicSpace::remainder(1), Some(1));
    assert_eq!(OnlyDynamicSpace::remainder(u32::MAX), Some(u32::MAX));

    assert_eq!(
        OnlyDynamicSpace::try_from(0).ok(),
        Some(OnlyDynamicSpace::Dynamic(0))
    );
    assert_eq!(
        OnlyDynamicSpace::try_from(1000).ok(),
        Some(OnlyDynamicSpace::Dynamic(1000))
    );
    assert_eq!(
        OnlyDynamicSpace::try_from(u32::MAX).ok(),
        Some(OnlyDynamicSpace::Dynamic(u32::MAX))
    );
}

#[test]
fn test_large_values_and_overflow_protection() {
    // Test with large values to ensure no overflow
    let large_val = u32::MAX - 100;

    // These should all return None due to being out of range
    assert_eq!(
        ComplexAlternatingSpace::try_as::<FirstName>(large_val),
        None
    );
    assert_eq!(
        ComplexAlternatingSpace::try_as::<FirstRange>(large_val),
        None
    );
    assert_eq!(
        ComplexAlternatingSpace::try_as::<SecondName>(large_val),
        None
    );
    assert_eq!(
        ComplexAlternatingSpace::try_as::<SecondRange>(large_val),
        None
    );

    // But dynamic should work
    assert_eq!(
        ComplexAlternatingSpace::remainder(large_val),
        Some(large_val - 10)
    );
    assert_eq!(
        ComplexAlternatingSpace::try_from(large_val).ok(),
        Some(ComplexAlternatingSpace::Dynamic(large_val - 10))
    );
}

#[test]
fn test_round_trips_complex_layout() {
    // Test round trips for each token type in complex layout

    // FirstName round trips
    assert_eq!(ComplexAlternatingSpace::position_of(FirstName::A), 0);
    assert_eq!(ComplexAlternatingSpace::position_of(FirstName::B), 1);

    // FirstRange round trips
    assert_eq!(ComplexAlternatingSpace::position_of(FirstRange(0)), 2);
    assert_eq!(ComplexAlternatingSpace::position_of(FirstRange(1)), 3);
    assert_eq!(ComplexAlternatingSpace::position_of(FirstRange(2)), 4);
    // FirstRange(3) would be out of bounds for the token itself

    // SecondName round trips
    assert_eq!(ComplexAlternatingSpace::position_of(SecondName::X), 5);
    assert_eq!(ComplexAlternatingSpace::position_of(SecondName::Y), 6);
    assert_eq!(ComplexAlternatingSpace::position_of(SecondName::Z), 7);

    // SecondRange round trips
    assert_eq!(ComplexAlternatingSpace::position_of(SecondRange(0)), 8);
    assert_eq!(ComplexAlternatingSpace::position_of(SecondRange(1)), 9);
    // SecondRange(2) would be out of bounds for the token itself

    // Test that we can go global -> local -> global and get the same value
    for global_pos in [2u32, 3, 4, 5, 6, 7, 8, 9] {
        if let Some(decoded) = ComplexAlternatingSpace::try_from(global_pos).ok() {
            match decoded {
                ComplexAlternatingSpace::FirstRange(FirstRange(local)) => {
                    let back_global = ComplexAlternatingSpace::position_of(FirstRange(local));
                    assert_eq!(back_global, global_pos);
                }
                ComplexAlternatingSpace::SecondRange(SecondRange(local)) => {
                    let back_global = ComplexAlternatingSpace::position_of(SecondRange(local));
                    assert_eq!(back_global, global_pos);
                }
                _ => {} // NameTokens don't have local offsets to test
            }
        }
    }
}
