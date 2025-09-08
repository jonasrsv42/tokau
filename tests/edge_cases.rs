use tokau::{Name, NameToken, Position, RangeToken, Space, TokenSpace, range};

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
    assert_eq!(MinimalSpace::is::<TinyToken>(0), Some(TinyToken::Only));
    assert_eq!(MinimalSpace::is::<TinyToken>(1), None); // Just outside
    
    assert_eq!(MinimalSpace::decode(0), Some(MinimalSpace::Tiny(TinyToken::Only)));
    assert_eq!(MinimalSpace::decode(1), None);
    
    // Test round trip
    assert_eq!(TinyToken::Only.inside::<MinimalSpace>(), 0);
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
    assert_eq!(BoundarySpace::is::<SingleRangeToken>(0), Some(SingleRangeToken(0)));
    assert_eq!(BoundarySpace::is::<SingleRangeToken>(1), None); // Now in TinyToken range
    
    // Test TinyToken boundaries  
    assert_eq!(BoundarySpace::is::<TinyToken>(1), Some(TinyToken::Only));
    assert_eq!(BoundarySpace::is::<TinyToken>(2), None); // Out of space
    assert_eq!(BoundarySpace::is::<TinyToken>(0), None); // In SingleRangeToken range
    
    // Test decode at boundaries
    assert_eq!(BoundarySpace::decode(0), Some(BoundarySpace::Single(SingleRangeToken(0))));
    assert_eq!(BoundarySpace::decode(1), Some(BoundarySpace::Tiny(TinyToken::Only)));
    assert_eq!(BoundarySpace::decode(2), None);
    
    // Test round trips
    assert_eq!(SingleRangeToken::inside::<BoundarySpace>(0), Some(0));
    assert_eq!(SingleRangeToken::inside::<BoundarySpace>(1), None); // Out of bounds
    assert_eq!(TinyToken::Only.inside::<BoundarySpace>(), 1);
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
    assert_eq!(<ComplexAlternatingSpace as Position<SecondRange>>::OFFSET, 8);
    assert_eq!(ComplexAlternatingSpace::RESERVED, 10);
    
    // Test each section's boundaries
    
    // FirstName: 0..2
    assert_eq!(ComplexAlternatingSpace::is::<FirstName>(0), Some(FirstName::A));
    assert_eq!(ComplexAlternatingSpace::is::<FirstName>(1), Some(FirstName::B));
    assert_eq!(ComplexAlternatingSpace::is::<FirstName>(2), None);
    
    // FirstRange: 2..5
    assert_eq!(ComplexAlternatingSpace::is::<FirstRange>(2), Some(FirstRange(0)));
    assert_eq!(ComplexAlternatingSpace::is::<FirstRange>(3), Some(FirstRange(1)));
    assert_eq!(ComplexAlternatingSpace::is::<FirstRange>(4), Some(FirstRange(2)));
    assert_eq!(ComplexAlternatingSpace::is::<FirstRange>(5), None);
    assert_eq!(ComplexAlternatingSpace::is::<FirstRange>(1), None); // In FirstName range
    
    // SecondName: 5..8
    assert_eq!(ComplexAlternatingSpace::is::<SecondName>(5), Some(SecondName::X));
    assert_eq!(ComplexAlternatingSpace::is::<SecondName>(6), Some(SecondName::Y));
    assert_eq!(ComplexAlternatingSpace::is::<SecondName>(7), Some(SecondName::Z));
    assert_eq!(ComplexAlternatingSpace::is::<SecondName>(8), None);
    assert_eq!(ComplexAlternatingSpace::is::<SecondName>(4), None); // In FirstRange range
    
    // SecondRange: 8..10
    assert_eq!(ComplexAlternatingSpace::is::<SecondRange>(8), Some(SecondRange(0)));
    assert_eq!(ComplexAlternatingSpace::is::<SecondRange>(9), Some(SecondRange(1)));
    assert_eq!(ComplexAlternatingSpace::is::<SecondRange>(10), None); // Now dynamic
    assert_eq!(ComplexAlternatingSpace::is::<SecondRange>(7), None); // In SecondName range
    
    // Dynamic: 10+
    assert_eq!(ComplexAlternatingSpace::remainder(9), None); // Still in static range
    assert_eq!(ComplexAlternatingSpace::remainder(10), Some(0));
    assert_eq!(ComplexAlternatingSpace::remainder(100), Some(90));
    
    // Test decode across all ranges
    assert_eq!(ComplexAlternatingSpace::decode(0), Some(ComplexAlternatingSpace::FirstName(FirstName::A)));
    assert_eq!(ComplexAlternatingSpace::decode(2), Some(ComplexAlternatingSpace::FirstRange(FirstRange(0))));
    assert_eq!(ComplexAlternatingSpace::decode(5), Some(ComplexAlternatingSpace::SecondName(SecondName::X)));
    assert_eq!(ComplexAlternatingSpace::decode(8), Some(ComplexAlternatingSpace::SecondRange(SecondRange(0))));
    assert_eq!(ComplexAlternatingSpace::decode(10), Some(ComplexAlternatingSpace::Dynamic(0)));
    assert_eq!(ComplexAlternatingSpace::decode(100), Some(ComplexAlternatingSpace::Dynamic(90)));
}

#[test]
fn test_only_dynamic_space() {
    // OnlyDynamicSpace has RESERVED = 0, everything is dynamic
    assert_eq!(OnlyDynamicSpace::RESERVED, 0);
    
    assert_eq!(OnlyDynamicSpace::remainder(0), Some(0));
    assert_eq!(OnlyDynamicSpace::remainder(1), Some(1));
    assert_eq!(OnlyDynamicSpace::remainder(u32::MAX), Some(u32::MAX));
    
    assert_eq!(OnlyDynamicSpace::decode(0), Some(OnlyDynamicSpace::Dynamic(0)));
    assert_eq!(OnlyDynamicSpace::decode(1000), Some(OnlyDynamicSpace::Dynamic(1000)));
    assert_eq!(OnlyDynamicSpace::decode(u32::MAX), Some(OnlyDynamicSpace::Dynamic(u32::MAX)));
}

#[test]
fn test_large_values_and_overflow_protection() {
    // Test with large values to ensure no overflow
    let large_val = u32::MAX - 100;
    
    // These should all return None due to being out of range
    assert_eq!(ComplexAlternatingSpace::is::<FirstName>(large_val), None);
    assert_eq!(ComplexAlternatingSpace::is::<FirstRange>(large_val), None);
    assert_eq!(ComplexAlternatingSpace::is::<SecondName>(large_val), None);
    assert_eq!(ComplexAlternatingSpace::is::<SecondRange>(large_val), None);
    
    // But dynamic should work
    assert_eq!(ComplexAlternatingSpace::remainder(large_val), Some(large_val - 10));
    assert_eq!(ComplexAlternatingSpace::decode(large_val), Some(ComplexAlternatingSpace::Dynamic(large_val - 10)));
}

#[test]
fn test_round_trips_complex_layout() {
    // Test round trips for each token type in complex layout
    
    // FirstName round trips
    assert_eq!(FirstName::A.inside::<ComplexAlternatingSpace>(), 0);
    assert_eq!(FirstName::B.inside::<ComplexAlternatingSpace>(), 1);
    
    // FirstRange round trips  
    assert_eq!(FirstRange::inside::<ComplexAlternatingSpace>(0), Some(2));
    assert_eq!(FirstRange::inside::<ComplexAlternatingSpace>(1), Some(3));
    assert_eq!(FirstRange::inside::<ComplexAlternatingSpace>(2), Some(4));
    assert_eq!(FirstRange::inside::<ComplexAlternatingSpace>(3), None); // Out of bounds
    
    // SecondName round trips
    assert_eq!(SecondName::X.inside::<ComplexAlternatingSpace>(), 5);
    assert_eq!(SecondName::Y.inside::<ComplexAlternatingSpace>(), 6);
    assert_eq!(SecondName::Z.inside::<ComplexAlternatingSpace>(), 7);
    
    // SecondRange round trips
    assert_eq!(SecondRange::inside::<ComplexAlternatingSpace>(0), Some(8));
    assert_eq!(SecondRange::inside::<ComplexAlternatingSpace>(1), Some(9));
    assert_eq!(SecondRange::inside::<ComplexAlternatingSpace>(2), None); // Out of bounds
    
    // Test that we can go global -> local -> global and get the same value
    for global_pos in [2u32, 3, 4, 5, 6, 7, 8, 9] {
        if let Some(decoded) = ComplexAlternatingSpace::decode(global_pos) {
            match decoded {
                ComplexAlternatingSpace::FirstRange(FirstRange(local)) => {
                    if let Some(back_global) = FirstRange::inside::<ComplexAlternatingSpace>(local) {
                        assert_eq!(back_global, global_pos);
                    }
                }
                ComplexAlternatingSpace::SecondRange(SecondRange(local)) => {
                    if let Some(back_global) = SecondRange::inside::<ComplexAlternatingSpace>(local) {
                        assert_eq!(back_global, global_pos);
                    }
                }
                _ => {} // NameTokens don't have local offsets to test
            }
        }
    }
}
