use tokau::{Name, NameToken, Position, RangeToken, Space, TokauError, Token, TokenSpace, range};

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

#[derive(Debug, PartialEq)]
#[range(100)]
struct TextRange(u32);

#[derive(Debug, PartialEq)]
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
    assert_eq!(CommonToken::Alpha.inside::<FirstSpace>(), 0);
    assert_eq!(CommonToken::Alpha.inside::<SecondSpace>(), 52);
    assert_eq!(CommonToken::Alpha.inside::<ThirdSpace>(), 150);
    assert_eq!(CommonToken::Alpha.inside::<MinimalSpace>(), 0);

    // Test CommonToken::Gamma in different spaces
    assert_eq!(CommonToken::Gamma.inside::<FirstSpace>(), 2);
    assert_eq!(CommonToken::Gamma.inside::<SecondSpace>(), 54);
    assert_eq!(CommonToken::Gamma.inside::<ThirdSpace>(), 152);
    assert_eq!(CommonToken::Gamma.inside::<MinimalSpace>(), 2);

    // Test SpecialToken::Start in different spaces
    assert_eq!(SpecialToken::Start.inside::<FirstSpace>(), 3);
    assert_eq!(SpecialToken::Start.inside::<SecondSpace>(), 0);
    assert_eq!(SpecialToken::Start.inside::<ThirdSpace>(), 153);

    // Test SpecialToken::End in different spaces
    assert_eq!(SpecialToken::End.inside::<FirstSpace>(), 4);
    assert_eq!(SpecialToken::End.inside::<SecondSpace>(), 1);
    assert_eq!(SpecialToken::End.inside::<ThirdSpace>(), 154);
}

#[test]
fn test_range_token_inside_different_spaces() {
    // Test TextRange in different spaces
    assert_eq!(TextRange::inside::<FirstSpace>(0), Some(5));
    assert_eq!(TextRange::inside::<FirstSpace>(99), Some(104));
    assert_eq!(TextRange::inside::<FirstSpace>(100), None); // Out of bounds

    assert_eq!(TextRange::inside::<ThirdSpace>(0), Some(0));
    assert_eq!(TextRange::inside::<ThirdSpace>(99), Some(99));
    assert_eq!(TextRange::inside::<ThirdSpace>(100), None); // Out of bounds

    // Test AudioRange in different spaces
    assert_eq!(AudioRange::inside::<SecondSpace>(0), Some(2));
    assert_eq!(AudioRange::inside::<SecondSpace>(49), Some(51));
    assert_eq!(AudioRange::inside::<SecondSpace>(50), None); // Out of bounds

    assert_eq!(AudioRange::inside::<ThirdSpace>(0), Some(100));
    assert_eq!(AudioRange::inside::<ThirdSpace>(49), Some(149));
    assert_eq!(AudioRange::inside::<ThirdSpace>(50), None); // Out of bounds
}

#[test]
fn test_reverse_lookups_across_spaces() {
    // Test CommonToken lookups in FirstSpace
    assert_eq!(FirstSpace::is::<CommonToken>(0), Some(CommonToken::Alpha));
    assert_eq!(FirstSpace::is::<CommonToken>(1), Some(CommonToken::Beta));
    assert_eq!(FirstSpace::is::<CommonToken>(2), Some(CommonToken::Gamma));
    assert_eq!(FirstSpace::is::<CommonToken>(3), None); // SpecialToken range

    // Test CommonToken lookups in SecondSpace (different positions)
    assert_eq!(SecondSpace::is::<CommonToken>(52), Some(CommonToken::Alpha));
    assert_eq!(SecondSpace::is::<CommonToken>(53), Some(CommonToken::Beta));
    assert_eq!(SecondSpace::is::<CommonToken>(54), Some(CommonToken::Gamma));
    assert_eq!(SecondSpace::is::<CommonToken>(0), None); // SpecialToken range
    assert_eq!(SecondSpace::is::<CommonToken>(51), None); // AudioRange

    // Test SpecialToken lookups in different spaces
    assert_eq!(FirstSpace::is::<SpecialToken>(3), Some(SpecialToken::Start));
    assert_eq!(FirstSpace::is::<SpecialToken>(4), Some(SpecialToken::End));
    assert_eq!(FirstSpace::is::<SpecialToken>(0), None); // CommonToken range

    assert_eq!(
        SecondSpace::is::<SpecialToken>(0),
        Some(SpecialToken::Start)
    );
    assert_eq!(SecondSpace::is::<SpecialToken>(1), Some(SpecialToken::End));
    assert_eq!(SecondSpace::is::<SpecialToken>(3), None); // AudioRange

    // Test range token lookups
    assert_eq!(FirstSpace::is::<TextRange>(5), Some(TextRange(0)));
    assert_eq!(FirstSpace::is::<TextRange>(104), Some(TextRange(99)));
    assert_eq!(FirstSpace::is::<TextRange>(0), None); // CommonToken range

    assert_eq!(ThirdSpace::is::<TextRange>(0), Some(TextRange(0)));
    assert_eq!(ThirdSpace::is::<TextRange>(99), Some(TextRange(99)));
    assert_eq!(ThirdSpace::is::<TextRange>(100), None); // AudioRange
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

    // But TryFrom should fail for values beyond static range when no dynamic variant exists
    assert_eq!(
        ThirdSpace::try_from(154).ok(),
        Some(ThirdSpace::Special(SpecialToken::End))
    ); // Last valid static
    assert_eq!(ThirdSpace::try_from(155).ok(), None); // Beyond static range, no dynamic variant
    assert_eq!(ThirdSpace::try_from(1000).ok(), None); // Way beyond static range, still no dynamic variant

    // Verify the error type is correct
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
fn test_round_trip_consistency() {
    // Test that global -> token -> global gives same result for each space

    // Test CommonToken round trips in different spaces
    let common_tokens = [CommonToken::Alpha, CommonToken::Beta, CommonToken::Gamma];

    for token in common_tokens {
        // FirstSpace
        let global_pos = token.inside::<FirstSpace>();
        assert_eq!(FirstSpace::is::<CommonToken>(global_pos), Some(token));

        // SecondSpace
        let global_pos = token.inside::<SecondSpace>();
        assert_eq!(SecondSpace::is::<CommonToken>(global_pos), Some(token));

        // ThirdSpace
        let global_pos = token.inside::<ThirdSpace>();
        assert_eq!(ThirdSpace::is::<CommonToken>(global_pos), Some(token));

        // MinimalSpace
        let global_pos = token.inside::<MinimalSpace>();
        assert_eq!(MinimalSpace::is::<CommonToken>(global_pos), Some(token));
    }

    // Test SpecialToken round trips
    let special_tokens = [SpecialToken::Start, SpecialToken::End];

    for token in special_tokens {
        // FirstSpace
        let global_pos = token.inside::<FirstSpace>();
        assert_eq!(FirstSpace::is::<SpecialToken>(global_pos), Some(token));

        // SecondSpace
        let global_pos = token.inside::<SecondSpace>();
        assert_eq!(SecondSpace::is::<SpecialToken>(global_pos), Some(token));

        // ThirdSpace
        let global_pos = token.inside::<ThirdSpace>();
        assert_eq!(ThirdSpace::is::<SpecialToken>(global_pos), Some(token));
    }

    // Test range token round trips for a few values
    for local_pos in [0, 25, 49, 99] {
        // TextRange in FirstSpace
        if let Some(global_pos) = TextRange::inside::<FirstSpace>(local_pos) {
            if let Some(TextRange(recovered_local)) = FirstSpace::is::<TextRange>(global_pos) {
                assert_eq!(recovered_local, local_pos);
            }
        }

        // TextRange in ThirdSpace
        if let Some(global_pos) = TextRange::inside::<ThirdSpace>(local_pos) {
            if let Some(TextRange(recovered_local)) = ThirdSpace::is::<TextRange>(global_pos) {
                assert_eq!(recovered_local, local_pos);
            }
        }

        // AudioRange in SecondSpace
        if local_pos < 50 {
            // AudioRange only goes to 50
            if let Some(global_pos) = AudioRange::inside::<SecondSpace>(local_pos) {
                if let Some(AudioRange(recovered_local)) = SecondSpace::is::<AudioRange>(global_pos)
                {
                    assert_eq!(recovered_local, local_pos);
                }
            }

            // AudioRange in ThirdSpace
            if let Some(global_pos) = AudioRange::inside::<ThirdSpace>(local_pos) {
                if let Some(AudioRange(recovered_local)) = ThirdSpace::is::<AudioRange>(global_pos)
                {
                    assert_eq!(recovered_local, local_pos);
                }
            }
        }
    }
}

#[test]
fn test_cross_space_isolation() {
    // Verify that tokens from one space don't interfere with another

    // A position that's CommonToken in FirstSpace should not be CommonToken in SecondSpace
    assert_eq!(FirstSpace::is::<CommonToken>(0), Some(CommonToken::Alpha));
    assert_eq!(SecondSpace::is::<CommonToken>(0), None); // This is SpecialToken in SecondSpace

    // A position that's SpecialToken in SecondSpace should not be SpecialToken in FirstSpace
    assert_eq!(
        SecondSpace::is::<SpecialToken>(0),
        Some(SpecialToken::Start)
    );
    assert_eq!(FirstSpace::is::<SpecialToken>(0), None); // This is CommonToken in FirstSpace

    // Verify range isolation
    assert_eq!(FirstSpace::is::<TextRange>(5), Some(TextRange(0)));
    assert_eq!(SecondSpace::is::<AudioRange>(5), Some(AudioRange(3))); // This is AudioRange in SecondSpace

    // Test that high positions in one space don't leak into another
    assert_eq!(FirstSpace::is::<CommonToken>(52), None); // This would be CommonToken in SecondSpace
    assert_eq!(SecondSpace::is::<CommonToken>(2), None); // This would be CommonToken in FirstSpace
}
