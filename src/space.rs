use crate::error::TokauError;
use crate::token::Token;

/// Macro to get compile-time token position in a space
/// Usage: `const_position!(Space, Token::Variant)`
///
/// This macro exists as a workaround for the lack of const fn in traits in stable Rust.
/// Once const fn in traits is stabilized, this could be replaced with a const version
/// of the `position_of` method.
#[macro_export]
macro_rules! const_position {
    ($space:ty, $token_type:ident :: $variant:ident) => {
        <$space as $crate::Position<$token_type>>::OFFSET + $token_type::$variant as u32
    };
}

pub trait Position<TokenType: Token> {
    const OFFSET: u32;

    // For Token instances - convert instance to global position
    fn at(token: TokenType) -> u32 {
        token.value() + Self::OFFSET
    }
}

pub trait TokenSpace: Sized + TryFrom<u32, Error = TokauError> {
    const RESERVED: u32; // Fixed/static part of the token space

    /// Convert a Space instance back to its global position value
    fn value(self) -> u32
    where
        Self: Copy;

    // For NameToken tokens - try to convert global value back to token instance
    fn try_as<T: Token>(value: u32) -> Option<T>
    where
        Self: Position<T>,
        T: TryFrom<u32, Error = TokauError>,
    {
        let start = <Self as Position<T>>::OFFSET;
        value.checked_sub(start).and_then(|v| T::try_from(v).ok())
    }

    // Get the global position of any Token in this space
    fn position_of<T: Token>(token: T) -> u32
    where
        Self: Position<T>,
    {
        <Self as Position<T>>::at(token)
    }

    // Return remainders outside reserved range, this can
    // overlap and exceed any dynamic vocabulary.
    fn remainder(value: u32) -> Option<u32> {
        value.checked_sub(Self::RESERVED)
    }

    // Check if a token is within reserved range
    fn is_reserved(value: u32) -> bool {
        value < Self::RESERVED
    }

    // Shift a value to after the reserved range
    fn after_reserved(value: u32) -> u32 {
        value + Self::RESERVED
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::token::Token;
    use crate::token::tests::*;

    impl Position<MaoToken> for GingerSpace {
        const OFFSET: u32 = GingerToken::COUNT;
    }

    impl Position<GingerToken> for GingerSpace {
        const OFFSET: u32 = 0;
    }

    impl Position<SingleToken> for GingerSpace {
        const OFFSET: u32 = GingerToken::COUNT + MaoToken::COUNT;
    }

    impl Position<TextTokens> for GingerSpace {
        const OFFSET: u32 = GingerToken::COUNT + MaoToken::COUNT + SingleToken::COUNT;
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub(crate) enum GingerSpace {
        Ginger(GingerToken),
        Mao(MaoToken),
        Single(SingleToken),
        Text(TextTokens),
    }

    impl TokenSpace for GingerSpace {
        const RESERVED: u32 =
            GingerToken::COUNT + MaoToken::COUNT + SingleToken::COUNT + TextTokens::COUNT;

        fn value(self) -> u32 {
            match self {
                GingerSpace::Ginger(token) => Self::position_of(token),
                GingerSpace::Mao(token) => Self::position_of(token),
                GingerSpace::Single(token) => Self::position_of(token),
                GingerSpace::Text(token) => Self::position_of(token),
            }
        }
    }

    impl TryFrom<u32> for GingerSpace {
        type Error = TokauError;

        fn try_from(id: u32) -> Result<Self, Self::Error> {
            // Match-based approach - compiler can optimize to jump table
            match id {
                // GingerToken range: 0..5
                0..=4 => GingerToken::try_from(id).ok().map(GingerSpace::Ginger),
                // MaoToken range: 5..9
                5..=8 => MaoToken::try_from(id - 5).ok().map(GingerSpace::Mao),
                // SingleToken range: 9..10
                9 => SingleToken::try_from(id - 9).ok().map(GingerSpace::Single),
                // TextTokens range: 10..1010
                10..=1009 => TextTokens::try_from(id - 10).ok().map(GingerSpace::Text),
                _ => None,
            }
            .ok_or(TokauError::OutOfRange {
                value: id,
                max: Self::RESERVED,
            })
        }
    }

    // Example of a dynamic token space
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub(crate) enum DynamicGingerSpace {
        Ginger(GingerToken),
        Mao(MaoToken),
        Single(SingleToken),
        Text(TextTokens),
        Dynamic(u32), // Dynamic vocabulary tokens
    }

    impl Position<MaoToken> for DynamicGingerSpace {
        const OFFSET: u32 = GingerToken::COUNT;
    }

    impl Position<GingerToken> for DynamicGingerSpace {
        const OFFSET: u32 = 0;
    }

    impl Position<SingleToken> for DynamicGingerSpace {
        const OFFSET: u32 = GingerToken::COUNT + MaoToken::COUNT;
    }

    impl Position<TextTokens> for DynamicGingerSpace {
        const OFFSET: u32 = GingerToken::COUNT + MaoToken::COUNT + SingleToken::COUNT;
    }

    impl TokenSpace for DynamicGingerSpace {
        const RESERVED: u32 =
            GingerToken::COUNT + MaoToken::COUNT + SingleToken::COUNT + TextTokens::COUNT;

        fn value(self) -> u32 {
            match self {
                DynamicGingerSpace::Ginger(token) => Self::position_of(token),
                DynamicGingerSpace::Mao(token) => Self::position_of(token),
                DynamicGingerSpace::Single(token) => Self::position_of(token),
                DynamicGingerSpace::Text(token) => Self::position_of(token),
                DynamicGingerSpace::Dynamic(offset) => Self::RESERVED + offset,
            }
        }
    }

    impl TryFrom<u32> for DynamicGingerSpace {
        type Error = TokauError;

        fn try_from(id: u32) -> Result<Self, Self::Error> {
            if let Some(token) = Self::try_as::<GingerToken>(id) {
                return Ok(DynamicGingerSpace::Ginger(token));
            }
            if let Some(token) = Self::try_as::<MaoToken>(id) {
                return Ok(DynamicGingerSpace::Mao(token));
            }
            if let Some(token) = Self::try_as::<SingleToken>(id) {
                return Ok(DynamicGingerSpace::Single(token));
            }
            if let Some(token) = Self::try_as::<TextTokens>(id) {
                return Ok(DynamicGingerSpace::Text(token));
            }
            if let Some(offset) = Self::remainder(id) {
                return Ok(DynamicGingerSpace::Dynamic(offset));
            }
            // Since thtry_as has dynamic tokens, there's no real upper bound
            // We never actually return an error for DynamicGingerSpace
            unreachable!("DynamicGingerSpace accepts all values via dynamic tokens")
        }
    }

    #[test]
    fn test_accessing_tokens_in_space() {
        assert_eq!(GingerSpace::position_of(GingerToken::TextStart), 0);
        assert_eq!(GingerSpace::position_of(MaoToken::ProgramStart), 5);
        assert_eq!(GingerSpace::position_of(SingleToken::Single), 9);

        let mao_fn = GingerSpace::position_of(MaoToken::Fn);
        assert_eq!(mao_fn, 7);

        let ginger_audio = GingerSpace::position_of(GingerToken::AudioStart);
        assert_eq!(ginger_audio, 2);
    }

    #[test]
    fn test_try_as_token_in_space() {
        // Check if value 5 try_as a MaoToken (should be ProgramStart)
        assert_eq!(
            GingerSpace::try_as::<MaoToken>(5),
            Some(MaoToken::ProgramStart)
        );
        assert_eq!(
            GingerSpace::try_as::<MaoToken>(6),
            Some(MaoToken::ProgramEnd)
        );
        assert_eq!(GingerSpace::try_as::<MaoToken>(7), Some(MaoToken::Fn));
        assert_eq!(GingerSpace::try_as::<MaoToken>(8), Some(MaoToken::Struct));

        // Check if value 0 try_as a GingerToken (should be TextStart)
        assert_eq!(
            GingerSpace::try_as::<GingerToken>(0),
            Some(GingerToken::TextStart)
        );
        assert_eq!(
            GingerSpace::try_as::<GingerToken>(4),
            Some(GingerToken::AwaitAudio)
        );

        // Check SingleToken
        assert_eq!(
            GingerSpace::try_as::<SingleToken>(9),
            Some(SingleToken::Single)
        );

        // Out of range tests
        assert!(GingerSpace::try_as::<MaoToken>(1000).is_none());
        assert!(GingerSpace::try_as::<MaoToken>(4).is_none()); // Thtry_as try_as a GingerToken
        assert!(GingerSpace::try_as::<GingerToken>(5).is_none()); // Thtry_as try_as a MaoToken
    }

    #[test]
    fn test_range_tokens() {
        // Test RangeToken position_of method
        assert_eq!(GingerSpace::position_of(TextTokens(0)), 10); // First position
        assert_eq!(GingerSpace::position_of(TextTokens(1)), 11); // Second position
        assert_eq!(GingerSpace::position_of(TextTokens(999)), 1009); // Last position
        // TextTokens(1000) would be out of bounds for the token itself
    }

    #[test]
    fn test_remainder_part() {
        assert_eq!(DynamicGingerSpace::remainder(1010), Some(0));
        assert_eq!(DynamicGingerSpace::remainder(1509), Some(499));
        assert_eq!(DynamicGingerSpace::remainder(500), None);

        // Static tokens still work
        assert_eq!(
            DynamicGingerSpace::try_as::<MaoToken>(5),
            Some(MaoToken::ProgramStart)
        );
    }

    #[test]
    fn test_try_as_with_range_tokens() {
        // Test try_as<T>() with RangeToken types (TextTokens)

        // Valid TextTokens in range [10, 1009]
        assert_eq!(GingerSpace::try_as::<TextTokens>(10), Some(TextTokens(0))); // First text token
        assert_eq!(GingerSpace::try_as::<TextTokens>(100), Some(TextTokens(90))); // Middle text token
        assert_eq!(
            GingerSpace::try_as::<TextTokens>(1009),
            Some(TextTokens(999))
        ); // Last text token

        // Out of range - should return None
        assert_eq!(GingerSpace::try_as::<TextTokens>(9), None); // Before range
        assert_eq!(GingerSpace::try_as::<TextTokens>(1010), None); // After range
        assert_eq!(GingerSpace::try_as::<TextTokens>(0), None); // In GingerToken range
        assert_eq!(GingerSpace::try_as::<TextTokens>(5), None); // In MaoToken range

        // Test with DynamicGingerSpace too
        assert_eq!(
            DynamicGingerSpace::try_as::<TextTokens>(10),
            Some(TextTokens(0))
        );
        assert_eq!(
            DynamicGingerSpace::try_as::<TextTokens>(1009),
            Some(TextTokens(999))
        );
        assert_eq!(DynamicGingerSpace::try_as::<TextTokens>(1010), None); // In dynamic range
    }

    #[test]
    fn test_decode_with_range_tokens() {
        // Test decode() method specifically for RangeToken types

        // TextTokens decoding in GingerSpace
        assert_eq!(
            GingerSpace::try_from(10).ok(),
            Some(GingerSpace::Text(TextTokens(0)))
        );
        assert_eq!(
            GingerSpace::try_from(100).ok(),
            Some(GingerSpace::Text(TextTokens(90)))
        );
        assert_eq!(
            GingerSpace::try_from(1009).ok(),
            Some(GingerSpace::Text(TextTokens(999)))
        );

        // Out of range should return None
        assert_eq!(GingerSpace::try_from(1010).ok(), None);

        // Test DynamicGingerSpace with both static and dynamic ranges
        assert_eq!(
            DynamicGingerSpace::try_from(10).ok(),
            Some(DynamicGingerSpace::Text(TextTokens(0)))
        );
        assert_eq!(
            DynamicGingerSpace::try_from(1009).ok(),
            Some(DynamicGingerSpace::Text(TextTokens(999)))
        );
        assert_eq!(
            DynamicGingerSpace::try_from(1010).ok(),
            Some(DynamicGingerSpace::Dynamic(0))
        ); // First dynamic
        assert_eq!(
            DynamicGingerSpace::try_from(2000).ok(),
            Some(DynamicGingerSpace::Dynamic(990))
        ); // Dynamic token
    }

    #[test]
    fn test_range_token_offset_calculations() {
        // Test that RangeToken offsets are calculated correctly when not at beginning

        // In GingerSpace, TextTokens starts at offset 10:
        // - GingerToken: 0..5 (5 tokens)
        // - MaoToken: 5..9 (4 tokens)
        // - SingleToken: 9..10 (1 token)
        // - TextTokens: 10..1010 (1000 tokens)

        // Test that global position correctly maps to local offset
        assert_eq!(GingerSpace::try_as::<TextTokens>(10), Some(TextTokens(0))); // Global 10 -> Local 0
        assert_eq!(GingerSpace::try_as::<TextTokens>(50), Some(TextTokens(40))); // Global 50 -> Local 40
        assert_eq!(
            GingerSpace::try_as::<TextTokens>(500),
            Some(TextTokens(490))
        ); // Global 500 -> Local 490
        assert_eq!(
            GingerSpace::try_as::<TextTokens>(1000),
            Some(TextTokens(990))
        ); // Global 1000 -> Local 990

        // Test that local offset correctly maps to global position using position_of
        assert_eq!(GingerSpace::position_of(TextTokens(0)), 10); // Local 0 -> Global 10
        assert_eq!(GingerSpace::position_of(TextTokens(40)), 50); // Local 40 -> Global 50
        assert_eq!(GingerSpace::position_of(TextTokens(490)), 500); // Local 490 -> Global 500 
        assert_eq!(GingerSpace::position_of(TextTokens(990)), 1000); // Local 990 -> Global 1000

        // Test round-trip: global -> local -> global
        let global_pos = 250u32;
        if let Some(local_token) = GingerSpace::try_as::<TextTokens>(global_pos) {
            assert_eq!(local_token, TextTokens(240)); // 250 - 10 = 240
            let back_to_global = GingerSpace::position_of(TextTokens(local_token.0));
            assert_eq!(back_to_global, global_pos); // Should get 250 back
        }
    }

    #[test]
    fn test_offset_calculations() {
        // Same token should have same value in both spaces (since they have same static layout)
        let mao_token = MaoToken::ProgramStart;
        let value_space1 = GingerSpace::position_of(mao_token);
        let value_space2 = DynamicGingerSpace::position_of(mao_token);
        assert_eq!(value_space1, value_space2);

        // Test that try_as() works correctly for both spaces
        assert_eq!(
            GingerSpace::try_as::<MaoToken>(5),
            Some(MaoToken::ProgramStart)
        );
        assert_eq!(
            DynamicGingerSpace::try_as::<MaoToken>(5),
            Some(MaoToken::ProgramStart)
        );
    }

    #[test]
    fn test_is_reserved() {
        // Test with GingerSpace (RESERVED = 1010)
        // Values below RESERVED should return true
        assert!(GingerSpace::is_reserved(0)); // First reserved token
        assert!(GingerSpace::is_reserved(1));
        assert!(GingerSpace::is_reserved(500)); // Middle of reserved range
        assert!(GingerSpace::is_reserved(1009)); // Last reserved token

        // Values at or above RESERVED should return false
        assert!(!GingerSpace::is_reserved(1010)); // First non-reserved value
        assert!(!GingerSpace::is_reserved(1011));
        assert!(!GingerSpace::is_reserved(2000));
        assert!(!GingerSpace::is_reserved(u32::MAX));

        // Test with DynamicGingerSpace (also RESERVED = 1010)
        assert!(DynamicGingerSpace::is_reserved(0));
        assert!(DynamicGingerSpace::is_reserved(999));
        assert!(DynamicGingerSpace::is_reserved(1009));
        assert!(!DynamicGingerSpace::is_reserved(1010));
        assert!(!DynamicGingerSpace::is_reserved(5000));

        // Edge cases around the boundary
        assert!(GingerSpace::is_reserved(1009)); // Last reserved
        assert!(!GingerSpace::is_reserved(1010)); // First non-reserved

        // Test specific token ranges to ensure they're all reserved
        // GingerToken range (0-4)
        assert!(GingerSpace::is_reserved(0));
        assert!(GingerSpace::is_reserved(4));

        // MaoToken range (5-8)
        assert!(GingerSpace::is_reserved(5));
        assert!(GingerSpace::is_reserved(8));

        // SingleToken (9)
        assert!(GingerSpace::is_reserved(9));

        // TextTokens range (10-1009)
        assert!(GingerSpace::is_reserved(10));
        assert!(GingerSpace::is_reserved(1009));
    }

    #[test]
    fn test_is_reserved_with_remainder() {
        // Test that is_reserved correctly identifies values that have remainders
        // Values with remainders should NOT be reserved

        // DynamicGingerSpace::RESERVED = 1010
        // remainder(1010) = Some(0), so 1010 is NOT reserved
        assert!(!DynamicGingerSpace::is_reserved(1010));
        assert_eq!(DynamicGingerSpace::remainder(1010), Some(0));

        // remainder(1500) = Some(490), so 1500 is NOT reserved
        assert!(!DynamicGingerSpace::is_reserved(1500));
        assert_eq!(DynamicGingerSpace::remainder(1500), Some(490));

        // remainder(500) = None (< RESERVED), so 500 IS reserved
        assert!(DynamicGingerSpace::is_reserved(500));
        assert_eq!(DynamicGingerSpace::remainder(500), None);

        // remainder(1009) = None (< RESERVED), so 1009 IS reserved
        assert!(DynamicGingerSpace::is_reserved(1009));
        assert_eq!(DynamicGingerSpace::remainder(1009), None);
    }

    #[test]
    fn test_reserved_boundary() {
        // Test the exact boundary between reserved and non-reserved tokens
        // GingerSpace::RESERVED = 1010

        // === Value 1009: Last reserved token ===
        // Should be reserved
        assert!(GingerSpace::is_reserved(1009));

        // Should decode successfully (it's the last TextToken)
        assert_eq!(
            GingerSpace::try_from(1009),
            Ok(GingerSpace::Text(TextTokens(999)))
        );

        // Should NOT have a remainder (it's within reserved range)
        assert_eq!(GingerSpace::remainder(1009), None);

        // === Value 1010: First non-reserved value ===
        // Should NOT be reserved
        assert!(!GingerSpace::is_reserved(1010));

        // Should FAIL to decode in GingerSpace (it's outside the static token range)
        assert!(GingerSpace::try_from(1010).is_err());

        // Should have a remainder of 0 (first value after reserved range)
        assert_eq!(GingerSpace::remainder(1010), Some(0));

        // === DynamicGingerSpace boundary test ===
        // 1009 behaves the same - it's reserved and decodable
        assert!(DynamicGingerSpace::is_reserved(1009));
        assert_eq!(
            DynamicGingerSpace::try_from(1009),
            Ok(DynamicGingerSpace::Text(TextTokens(999)))
        );
        assert_eq!(DynamicGingerSpace::remainder(1009), None);

        // 1010 is NOT reserved but CAN decode in DynamicGingerSpace (as Dynamic)
        assert!(!DynamicGingerSpace::is_reserved(1010));
        assert_eq!(
            DynamicGingerSpace::try_from(1010),
            Ok(DynamicGingerSpace::Dynamic(0)) // First dynamic token
        );
        assert_eq!(DynamicGingerSpace::remainder(1010), Some(0));

        // === Additional boundary values ===
        // 1008: Second-to-last reserved token
        assert!(GingerSpace::is_reserved(1008));
        assert_eq!(
            GingerSpace::try_from(1008),
            Ok(GingerSpace::Text(TextTokens(998)))
        );

        // 1011: Second non-reserved value
        assert!(!GingerSpace::is_reserved(1011));
        assert!(GingerSpace::try_from(1011).is_err());
        assert_eq!(GingerSpace::remainder(1011), Some(1));
    }

    #[test]
    fn test_const_position_macro() {
        // Test that the macro produces const values that can be used in match
        const GINGER_TEXT_START: u32 = const_position!(GingerSpace, GingerToken::TextStart);
        const GINGER_AUDIO_START: u32 = const_position!(GingerSpace, GingerToken::AudioStart);
        const MAO_PROGRAM_START: u32 = const_position!(GingerSpace, MaoToken::ProgramStart);
        const MAO_FN: u32 = const_position!(GingerSpace, MaoToken::Fn);

        // Verify these match runtime position_of
        assert_eq!(
            GINGER_TEXT_START,
            GingerSpace::position_of(GingerToken::TextStart)
        );
        assert_eq!(
            GINGER_AUDIO_START,
            GingerSpace::position_of(GingerToken::AudioStart)
        );
        assert_eq!(
            MAO_PROGRAM_START,
            GingerSpace::position_of(MaoToken::ProgramStart)
        );
        assert_eq!(MAO_FN, GingerSpace::position_of(MaoToken::Fn));

        // Test in match expression using the const values
        fn classify_token(id: u32) -> &'static str {
            const TEXT_START: u32 = const_position!(GingerSpace, GingerToken::TextStart);
            const AUDIO_START: u32 = const_position!(GingerSpace, GingerToken::AudioStart);
            const PROGRAM_START: u32 = const_position!(GingerSpace, MaoToken::ProgramStart);
            const FN: u32 = const_position!(GingerSpace, MaoToken::Fn);

            match id {
                TEXT_START => "TextStart",
                AUDIO_START => "AudioStart",
                PROGRAM_START => "ProgramStart",
                FN => "Fn",
                _ => "Other",
            }
        }

        assert_eq!(classify_token(0), "TextStart");
        assert_eq!(classify_token(2), "AudioStart");
        assert_eq!(classify_token(5), "ProgramStart");
        assert_eq!(classify_token(7), "Fn");
        assert_eq!(classify_token(100), "Other");
    }

    #[test]
    fn test_different_space_layouts() {
        use crate::space::{Position, TokenSpace};

        // Create a space with different offset layout using Space derive macro
        #[derive(Debug, PartialEq, Clone, Copy)]
        enum AlternativeSpace {
            Mao(MaoToken),       // Different order: Mao first
            Single(SingleToken), // Single second
            Ginger(GingerToken), // Ginger third
            Text(TextTokens),    // Text last
        }

        // Implement Position traits for the different ordering
        impl Position<MaoToken> for AlternativeSpace {
            const OFFSET: u32 = 0; // Mao tokens at start
        }

        impl Position<SingleToken> for AlternativeSpace {
            const OFFSET: u32 = MaoToken::COUNT; // Single after Mao
        }

        impl Position<GingerToken> for AlternativeSpace {
            const OFFSET: u32 = MaoToken::COUNT + SingleToken::COUNT; // Ginger after Single
        }

        impl Position<TextTokens> for AlternativeSpace {
            const OFFSET: u32 = MaoToken::COUNT + SingleToken::COUNT + GingerToken::COUNT; // Text last
        }

        impl TokenSpace for AlternativeSpace {
            const RESERVED: u32 =
                MaoToken::COUNT + SingleToken::COUNT + GingerToken::COUNT + TextTokens::COUNT;

            fn value(self) -> u32 {
                match self {
                    AlternativeSpace::Mao(token) => Self::position_of(token),
                    AlternativeSpace::Single(token) => Self::position_of(token),
                    AlternativeSpace::Ginger(token) => Self::position_of(token),
                    AlternativeSpace::Text(token) => Self::position_of(token),
                }
            }
        }

        impl TryFrom<u32> for AlternativeSpace {
            type Error = TokauError;

            fn try_from(id: u32) -> Result<Self, Self::Error> {
                if let Some(token) = Self::try_as::<MaoToken>(id) {
                    return Ok(AlternativeSpace::Mao(token));
                }
                if let Some(token) = Self::try_as::<SingleToken>(id) {
                    return Ok(AlternativeSpace::Single(token));
                }
                if let Some(token) = Self::try_as::<GingerToken>(id) {
                    return Ok(AlternativeSpace::Ginger(token));
                }
                if let Some(token) = Self::try_as::<TextTokens>(id) {
                    return Ok(AlternativeSpace::Text(token));
                }
                Err(TokauError::OutOfRange {
                    value: id,
                    max: Self::RESERVED,
                })
            }
        }

        // Test that same tokens have different values in different spaces
        let mao_in_dynamic = DynamicGingerSpace::position_of(MaoToken::ProgramStart);
        let mao_in_alt = AlternativeSpace::position_of(MaoToken::ProgramStart);

        // In DynamicGingerSpace: GingerToken(5) + MaoToken offset = 5 + 0 = 5
        // In AlternativeSpace: MaoToken offset = 0 + 0 = 0
        assert_eq!(mao_in_dynamic, 5);
        assert_eq!(mao_in_alt, 0);

        // Test filtering works correctly with different layouts
        let tokens = vec![0, 1, 5, 6, 7];

        // In AlternativeSpace, tokens 0,1,2,3 should be MaoTokens (since MaoToken starts at offset 0)
        let alt_maos: Vec<MaoToken> = tokens
            .clone()
            .into_iter()
            .filter_map(|id| AlternativeSpace::try_as::<MaoToken>(id))
            .collect();
        // Only tokens 0,1 are present in our test vector, so we get ProgramStart, ProgramEnd
        assert_eq!(alt_maos, vec![MaoToken::ProgramStart, MaoToken::ProgramEnd]);

        // In DynamicGingerSpace, token 5 should be MaoToken::ProgramStart
        let dyn_maos: Vec<MaoToken> = tokens
            .into_iter()
            .filter_map(|id| DynamicGingerSpace::try_as::<MaoToken>(id))
            .collect();
        assert_eq!(
            dyn_maos,
            vec![MaoToken::ProgramStart, MaoToken::ProgramEnd, MaoToken::Fn]
        );
    }

    #[test]
    fn test_after_reserved() {
        // Test shifting single values to after the reserved range

        // GingerSpace::RESERVED = 1010
        assert_eq!(GingerSpace::after_reserved(0), 1010);
        assert_eq!(GingerSpace::after_reserved(1), 1011);
        assert_eq!(GingerSpace::after_reserved(100), 1110);
        assert_eq!(GingerSpace::after_reserved(500), 1510);

        // Test that the shifted values are not reserved
        assert!(!GingerSpace::is_reserved(GingerSpace::after_reserved(0)));
        assert!(!GingerSpace::is_reserved(GingerSpace::after_reserved(100)));
        assert!(!GingerSpace::is_reserved(GingerSpace::after_reserved(500)));

        // Test that shifted values have remainders
        assert_eq!(
            GingerSpace::remainder(GingerSpace::after_reserved(0)),
            Some(0)
        );
        assert_eq!(
            GingerSpace::remainder(GingerSpace::after_reserved(100)),
            Some(100)
        );
        assert_eq!(
            GingerSpace::remainder(GingerSpace::after_reserved(500)),
            Some(500)
        );

        // Test round-trip: shift then remainder should give back original value
        let original = 42u32;
        let shifted = GingerSpace::after_reserved(original);
        let back_to_original = GingerSpace::remainder(shifted).unwrap();
        assert_eq!(back_to_original, original);

        // Test with DynamicGingerSpace too
        assert_eq!(DynamicGingerSpace::after_reserved(0), 1010);
        assert_eq!(DynamicGingerSpace::after_reserved(50), 1060);

        // Test edge case with u32::MAX - RESERVED to avoid overflow
        let max_safe = u32::MAX - DynamicGingerSpace::RESERVED;
        let shifted_max = DynamicGingerSpace::after_reserved(max_safe);
        assert_eq!(shifted_max, u32::MAX);
    }

    #[test]
    fn test_space_value() {
        // Test converting Space instances back to their global values

        // Test GingerSpace variants
        let ginger = GingerSpace::Ginger(GingerToken::TextStart);
        assert_eq!(ginger.value(), 0);

        let mao = GingerSpace::Mao(MaoToken::ProgramStart);
        assert_eq!(mao.value(), 5);

        let single = GingerSpace::Single(SingleToken::Single);
        assert_eq!(single.value(), 9);

        let text = GingerSpace::Text(TextTokens(0));
        assert_eq!(text.value(), 10);

        let text_high = GingerSpace::Text(TextTokens(999));
        assert_eq!(text_high.value(), 1009);

        // Test DynamicGingerSpace variants (same as GingerSpace for static tokens)
        let dyn_ginger = DynamicGingerSpace::Ginger(GingerToken::AudioStart);
        assert_eq!(dyn_ginger.value(), 2);

        let dyn_mao = DynamicGingerSpace::Mao(MaoToken::Fn);
        assert_eq!(dyn_mao.value(), 7);

        // Test dynamic tokens
        let dynamic_0 = DynamicGingerSpace::Dynamic(0);
        assert_eq!(dynamic_0.value(), 1010); // RESERVED + 0

        let dynamic_500 = DynamicGingerSpace::Dynamic(500);
        assert_eq!(dynamic_500.value(), 1510); // RESERVED + 500

        // Test round-trip: value -> try_from -> value
        let original_value = 42u32;
        if let Ok(space) = DynamicGingerSpace::try_from(original_value) {
            assert_eq!(space.value(), original_value);
        }

        // Test round-trip with high values
        let high_value = 2000u32;
        if let Ok(space) = DynamicGingerSpace::try_from(high_value) {
            assert_eq!(space.value(), high_value);
        }
    }
}
