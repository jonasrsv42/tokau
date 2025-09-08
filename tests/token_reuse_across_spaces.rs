use tokau::{Name, Position, Space, TokauError, Token, TokenSpace, range};

// Define reusable token types that will be used in multiple spaces
#[derive(Name, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
enum CommonToken {
    Alpha,
    Beta,
    Gamma,
}

#[derive(Name, Debug, PartialEq, Clone, Copy)]
#[repr(u32)]
enum SpecialToken {
    Start,
    End,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[range(100)]
struct TextRange(u32);

#[derive(Debug, PartialEq, Clone, Copy)]
#[range(50)]
struct AudioRange(u32);

// Define multiple spaces that reuse the same tokens in different positions
#[derive(Space, Debug, PartialEq)]
enum FirstSpace {
    Common(CommonToken),   // 0..3
    Special(SpecialToken), // 3..5
    Text(TextRange),       // 5..105
    #[dynamic]
    Dynamic(u32), // 105+
}

#[derive(Space, Debug, PartialEq)]
enum SecondSpace {
    Special(SpecialToken), // 0..2 (different position than FirstSpace)
    Audio(AudioRange),     // 2..52
    Common(CommonToken),   // 52..55 (different position than FirstSpace)
    #[dynamic]
    Dynamic(u32), // 55+
}

#[derive(Space, Debug, PartialEq)]
enum ThirdSpace {
    Text(TextRange),       // 0..100
    Audio(AudioRange),     // 100..150
    Common(CommonToken),   // 150..153 (yet another position)
    Special(SpecialToken), // 153..155 (yet another position)
}

#[derive(Space, Debug, PartialEq)]
enum MinimalSpace {
    Common(CommonToken), // 0..3 (same as FirstSpace but different overall layout)
}

#[test]
fn test_token_positions_across_spaces() {
    // Test CommonToken positions in different spaces
    assert_eq!(<FirstSpace as Position<CommonToken>>::OFFSET, 0);
    assert_eq!(<SecondSpace as Position<CommonToken>>::OFFSET, 52);
    assert_eq!(<ThirdSpace as Position<CommonToken>>::OFFSET, 150);
    assert_eq!(<MinimalSpace as Position<CommonToken>>::OFFSET, 0);

    // Test SpecialToken positions in different spaces
    assert_eq!(<FirstSpace as Position<SpecialToken>>::OFFSET, 3);
    assert_eq!(<SecondSpace as Position<SpecialToken>>::OFFSET, 0);
    assert_eq!(<ThirdSpace as Position<SpecialToken>>::OFFSET, 153);

    // Test range token positions
    assert_eq!(<FirstSpace as Position<TextRange>>::OFFSET, 5);
    assert_eq!(<ThirdSpace as Position<TextRange>>::OFFSET, 0);

    assert_eq!(<SecondSpace as Position<AudioRange>>::OFFSET, 2);
    assert_eq!(<ThirdSpace as Position<AudioRange>>::OFFSET, 100);
}

#[test]
fn test_token_inside_different_spaces() {
    // Test CommonToken::Alpha in different spaces
    assert_eq!(FirstSpace::position_of(CommonToken::Alpha), 0);
    assert_eq!(SecondSpace::position_of(CommonToken::Alpha), 52);
    assert_eq!(ThirdSpace::position_of(CommonToken::Alpha), 150);
    assert_eq!(MinimalSpace::position_of(CommonToken::Alpha), 0);

    // Test CommonToken::Gamma in different spaces
    assert_eq!(FirstSpace::position_of(CommonToken::Gamma), 2);
    assert_eq!(SecondSpace::position_of(CommonToken::Gamma), 54);
    assert_eq!(ThirdSpace::position_of(CommonToken::Gamma), 152);
    assert_eq!(MinimalSpace::position_of(CommonToken::Gamma), 2);

    // Test SpecialToken::Start in different spaces
    assert_eq!(FirstSpace::position_of(SpecialToken::Start), 3);
    assert_eq!(SecondSpace::position_of(SpecialToken::Start), 0);
    assert_eq!(ThirdSpace::position_of(SpecialToken::Start), 153);

    // Test SpecialToken::End in different spaces
    assert_eq!(FirstSpace::position_of(SpecialToken::End), 4);
    assert_eq!(SecondSpace::position_of(SpecialToken::End), 1);
    assert_eq!(ThirdSpace::position_of(SpecialToken::End), 154);
}

#[test]
fn test_range_token_inside_different_spaces() {
    // Test TextRange in different spaces
    assert_eq!(FirstSpace::position_of(TextRange(0)), 5);
    assert_eq!(FirstSpace::position_of(TextRange(99)), 104);
    // TextRange(100) would be out of bounds for the token itself

    assert_eq!(ThirdSpace::position_of(TextRange(0)), 0);
    assert_eq!(ThirdSpace::position_of(TextRange(99)), 99);
    // TextRange(100) would be out of bounds for the token itself

    // Test AudioRange in different spaces
    assert_eq!(SecondSpace::position_of(AudioRange(0)), 2);
    assert_eq!(SecondSpace::position_of(AudioRange(49)), 51);
    // AudioRange(50) would be out of bounds for the token itself

    assert_eq!(ThirdSpace::position_of(AudioRange(0)), 100);
    assert_eq!(ThirdSpace::position_of(AudioRange(49)), 149);
    // AudioRange(50) would be out of bounds for the token itself
}

#[test]
fn test_reverse_lookups_across_spaces() {
    // Test CommonToken lookups in FirstSpace
    assert_eq!(
        FirstSpace::try_as::<CommonToken>(0),
        Some(CommonToken::Alpha)
    );
    assert_eq!(
        FirstSpace::try_as::<CommonToken>(1),
        Some(CommonToken::Beta)
    );
    assert_eq!(
        FirstSpace::try_as::<CommonToken>(2),
        Some(CommonToken::Gamma)
    );
    assert_eq!(FirstSpace::try_as::<CommonToken>(3), None); // SpecialToken range

    // Test CommonToken lookups in SecondSpace (different positions)
    assert_eq!(
        SecondSpace::try_as::<CommonToken>(52),
        Some(CommonToken::Alpha)
    );
    assert_eq!(
        SecondSpace::try_as::<CommonToken>(53),
        Some(CommonToken::Beta)
    );
    assert_eq!(
        SecondSpace::try_as::<CommonToken>(54),
        Some(CommonToken::Gamma)
    );
    assert_eq!(SecondSpace::try_as::<CommonToken>(0), None); // SpecialToken range
    assert_eq!(SecondSpace::try_as::<CommonToken>(51), None); // AudioRange

    // Test SpecialToken lookups in different spaces
    assert_eq!(
        FirstSpace::try_as::<SpecialToken>(3),
        Some(SpecialToken::Start)
    );
    assert_eq!(
        FirstSpace::try_as::<SpecialToken>(4),
        Some(SpecialToken::End)
    );
    assert_eq!(FirstSpace::try_as::<SpecialToken>(0), None); // CommonToken range

    assert_eq!(
        SecondSpace::try_as::<SpecialToken>(0),
        Some(SpecialToken::Start)
    );
    assert_eq!(
        SecondSpace::try_as::<SpecialToken>(1),
        Some(SpecialToken::End)
    );
    assert_eq!(SecondSpace::try_as::<SpecialToken>(3), None); // AudioRange

    // Test range token lookups
    assert_eq!(FirstSpace::try_as::<TextRange>(5), Some(TextRange(0)));
    assert_eq!(FirstSpace::try_as::<TextRange>(104), Some(TextRange(99)));
    assert_eq!(FirstSpace::try_as::<TextRange>(0), None); // CommonToken range

    assert_eq!(ThirdSpace::try_as::<TextRange>(0), Some(TextRange(0)));
    assert_eq!(ThirdSpace::try_as::<TextRange>(99), Some(TextRange(99)));
    assert_eq!(ThirdSpace::try_as::<TextRange>(100), None); // AudioRange
}

#[test]
fn test_space_reserved_counts() {
    // Verify RESERVED calculations for each space
    assert_eq!(FirstSpace::RESERVED, 3 + 2 + 100); // 105
    assert_eq!(SecondSpace::RESERVED, 2 + 50 + 3); // 55  
    assert_eq!(ThirdSpace::RESERVED, 100 + 50 + 3 + 2); // 155
    assert_eq!(MinimalSpace::RESERVED, 3);
}

#[test]
fn test_decode_across_spaces() {
    // Test decoding in FirstSpace
    assert_eq!(
        FirstSpace::try_from(0).ok(),
        Some(FirstSpace::Common(CommonToken::Alpha))
    );
    assert_eq!(
        FirstSpace::try_from(2).ok(),
        Some(FirstSpace::Common(CommonToken::Gamma))
    );
    assert_eq!(
        FirstSpace::try_from(3).ok(),
        Some(FirstSpace::Special(SpecialToken::Start))
    );
    assert_eq!(
        FirstSpace::try_from(4).ok(),
        Some(FirstSpace::Special(SpecialToken::End))
    );
    assert_eq!(
        FirstSpace::try_from(5).ok(),
        Some(FirstSpace::Text(TextRange(0)))
    );
    assert_eq!(
        FirstSpace::try_from(104).ok(),
        Some(FirstSpace::Text(TextRange(99)))
    );
    assert_eq!(FirstSpace::try_from(105).ok(), Some(FirstSpace::Dynamic(0)));

    // Test decoding in SecondSpace (different layout)
    assert_eq!(
        SecondSpace::try_from(0).ok(),
        Some(SecondSpace::Special(SpecialToken::Start))
    );
    assert_eq!(
        SecondSpace::try_from(1).ok(),
        Some(SecondSpace::Special(SpecialToken::End))
    );
    assert_eq!(
        SecondSpace::try_from(2).ok(),
        Some(SecondSpace::Audio(AudioRange(0)))
    );
    assert_eq!(
        SecondSpace::try_from(51).ok(),
        Some(SecondSpace::Audio(AudioRange(49)))
    );
    assert_eq!(
        SecondSpace::try_from(52).ok(),
        Some(SecondSpace::Common(CommonToken::Alpha))
    );
    assert_eq!(
        SecondSpace::try_from(54).ok(),
        Some(SecondSpace::Common(CommonToken::Gamma))
    );
    assert_eq!(
        SecondSpace::try_from(55).ok(),
        Some(SecondSpace::Dynamic(0))
    );

    // Test decoding in ThirdSpace (no dynamic tokens)
    assert_eq!(
        ThirdSpace::try_from(0).ok(),
        Some(ThirdSpace::Text(TextRange(0)))
    );
    assert_eq!(
        ThirdSpace::try_from(99).ok(),
        Some(ThirdSpace::Text(TextRange(99)))
    );
    assert_eq!(
        ThirdSpace::try_from(100).ok(),
        Some(ThirdSpace::Audio(AudioRange(0)))
    );
    assert_eq!(
        ThirdSpace::try_from(149).ok(),
        Some(ThirdSpace::Audio(AudioRange(49)))
    );
    assert_eq!(
        ThirdSpace::try_from(150).ok(),
        Some(ThirdSpace::Common(CommonToken::Alpha))
    );
    assert_eq!(
        ThirdSpace::try_from(152).ok(),
        Some(ThirdSpace::Common(CommonToken::Gamma))
    );
    assert_eq!(
        ThirdSpace::try_from(153).ok(),
        Some(ThirdSpace::Special(SpecialToken::Start))
    );
    assert_eq!(
        ThirdSpace::try_from(154).ok(),
        Some(ThirdSpace::Special(SpecialToken::End))
    );
    assert_eq!(ThirdSpace::try_from(155).ok(), None); // Out of range (no dynamic)
}

#[test]
fn test_dynamic_tokens_in_different_spaces() {
    // FirstSpace dynamic tokens start at 105
    assert_eq!(FirstSpace::remainder(104), None); // Still in static range
    assert_eq!(FirstSpace::remainder(105), Some(0));
    assert_eq!(FirstSpace::remainder(200), Some(95));

    // SecondSpace dynamic tokens start at 55
    assert_eq!(SecondSpace::remainder(54), None); // Still in static range
    assert_eq!(SecondSpace::remainder(55), Some(0));
    assert_eq!(SecondSpace::remainder(100), Some(45));

    // ThirdSpace has no dynamic tokens, but remainder still works
    assert_eq!(ThirdSpace::remainder(154), None); // In static range
    assert_eq!(ThirdSpace::remainder(155), Some(0)); // Can compute remainder, but no decode
    assert_eq!(ThirdSpace::remainder(1000), Some(845)); // Can compute remainder, but no decode

    // But TryFrom should fail for values beyond static range when no dynamic variant extry_asts
    assert_eq!(
        ThirdSpace::try_from(154).ok(),
        Some(ThirdSpace::Special(SpecialToken::End))
    ); // Last valid static
    assert_eq!(ThirdSpace::try_from(155).ok(), None); // Beyond static range, no dynamic variant
    assert_eq!(ThirdSpace::try_from(1000).ok(), None); // Way beyond static range, still no dynamic variant

    // Verify the error type try_as correct
    assert_eq!(
        ThirdSpace::try_from(155),
        Err(TokauError::OutOfRange {
            value: 155,
            max: ThirdSpace::RESERVED
        })
    );
    assert_eq!(
        ThirdSpace::try_from(1000),
        Err(TokauError::OutOfRange {
            value: 1000,
            max: ThirdSpace::RESERVED
        })
    );
}

#[test]
fn test_round_trip_constry_astency() {
    // Test that global -> token -> global gives same result for each space

    // Test CommonToken round trips in different spaces
    let common_tokens = [CommonToken::Alpha, CommonToken::Beta, CommonToken::Gamma];

    for token in common_tokens {
        // FirstSpace
        let global_pos = FirstSpace::position_of(token);
        assert_eq!(FirstSpace::try_as::<CommonToken>(global_pos), Some(token));

        // SecondSpace
        let global_pos = SecondSpace::position_of(token);
        assert_eq!(SecondSpace::try_as::<CommonToken>(global_pos), Some(token));

        // ThirdSpace
        let global_pos = ThirdSpace::position_of(token);
        assert_eq!(ThirdSpace::try_as::<CommonToken>(global_pos), Some(token));

        // MinimalSpace
        let global_pos = MinimalSpace::position_of(token);
        assert_eq!(MinimalSpace::try_as::<CommonToken>(global_pos), Some(token));
    }

    // Test SpecialToken round trips
    let special_tokens = [SpecialToken::Start, SpecialToken::End];

    for token in special_tokens {
        // FirstSpace
        let global_pos = FirstSpace::position_of(token);
        assert_eq!(FirstSpace::try_as::<SpecialToken>(global_pos), Some(token));

        // SecondSpace
        let global_pos = SecondSpace::position_of(token);
        assert_eq!(SecondSpace::try_as::<SpecialToken>(global_pos), Some(token));

        // ThirdSpace
        let global_pos = ThirdSpace::position_of(token);
        assert_eq!(ThirdSpace::try_as::<SpecialToken>(global_pos), Some(token));
    }

    // Test range token round trips for a few values
    for local_pos in [0, 25, 49, 99] {
        // TextRange in FirstSpace
        if let Ok(token) = TextRange::try_from(local_pos) {
            let global_pos = FirstSpace::position_of(token);
            if let Some(TextRange(recovered_local)) = FirstSpace::try_as::<TextRange>(global_pos) {
                assert_eq!(recovered_local, local_pos);
            }
        }

        // TextRange in ThirdSpace
        if let Ok(token) = TextRange::try_from(local_pos) {
            let global_pos = ThirdSpace::position_of(token);
            if let Some(TextRange(recovered_local)) = ThirdSpace::try_as::<TextRange>(global_pos) {
                assert_eq!(recovered_local, local_pos);
            }
        }

        // AudioRange in SecondSpace
        if let Ok(token) = AudioRange::try_from(local_pos) {
            let global_pos = SecondSpace::position_of(token);
            if let Some(AudioRange(recovered_local)) = SecondSpace::try_as::<AudioRange>(global_pos)
            {
                assert_eq!(recovered_local, local_pos);
            }

            // AudioRange in ThirdSpace
            let global_pos = ThirdSpace::position_of(token);
            if let Some(AudioRange(recovered_local)) = ThirdSpace::try_as::<AudioRange>(global_pos)
            {
                assert_eq!(recovered_local, local_pos);
            }
        }
    }
}

#[test]
fn test_cross_space_try_asolation() {
    // Verify that tokens from one space don't interfere with another

    // A position that's CommonToken in FirstSpace should not be CommonToken in SecondSpace
    assert_eq!(
        FirstSpace::try_as::<CommonToken>(0),
        Some(CommonToken::Alpha)
    );
    assert_eq!(SecondSpace::try_as::<CommonToken>(0), None); // Thtry_as try_as SpecialToken in SecondSpace

    // A position that's SpecialToken in SecondSpace should not be SpecialToken in FirstSpace
    assert_eq!(
        SecondSpace::try_as::<SpecialToken>(0),
        Some(SpecialToken::Start)
    );
    assert_eq!(FirstSpace::try_as::<SpecialToken>(0), None); // Thtry_as try_as CommonToken in FirstSpace

    // Verify range try_asolation
    assert_eq!(FirstSpace::try_as::<TextRange>(5), Some(TextRange(0)));
    assert_eq!(SecondSpace::try_as::<AudioRange>(5), Some(AudioRange(3))); // Thtry_as try_as AudioRange in SecondSpace

    // Test that high positions in one space don't leak into another
    assert_eq!(FirstSpace::try_as::<CommonToken>(52), None); // Thtry_as would be CommonToken in SecondSpace
    assert_eq!(SecondSpace::try_as::<CommonToken>(2), None); // Thtry_as would be CommonToken in FirstSpace
}
