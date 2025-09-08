use crate::error::TokauError;
use crate::token::{NameToken, Token};

pub trait Position<TokenType: Token> {
    const OFFSET: u32;

    // For NameToken tokens - convert instance to global position
    fn at(token: &TokenType) -> u32
    where
        TokenType: NameToken,
    {
        token.value() + Self::OFFSET
    }
}

pub trait TokenSpace: Sized + TryFrom<u32, Error = TokauError> {
    const RESERVED: u32; // Fixed/static part of the token space

    // For NameToken tokens - try to convert global value back to token instance
    fn is<T: Token>(value: u32) -> Option<T>
    where
        Self: Position<T>,
        T: TryFrom<u32, Error = TokauError>,
    {
        let start = <Self as Position<T>>::OFFSET;
        value.checked_sub(start).and_then(|v| T::try_from(v).ok())
    }

    // Return remainders outside reserved range, this can
    // overlap and exceed any dynamic vocabulary.
    fn remainder(value: u32) -> Option<u32> {
        value.checked_sub(Self::RESERVED)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::token::tests::*;
    use crate::token::{NameToken, RangeToken};

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

    #[derive(Debug, PartialEq)]
    pub(crate) enum GingerSpace {
        Ginger(GingerToken),
        Mao(MaoToken),
        Single(SingleToken),
        Text(TextTokens),
    }

    impl TokenSpace for GingerSpace {
        const RESERVED: u32 =
            GingerToken::COUNT + MaoToken::COUNT + SingleToken::COUNT + TextTokens::COUNT;
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
    #[derive(Debug, PartialEq)]
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
    }

    impl TryFrom<u32> for DynamicGingerSpace {
        type Error = TokauError;

        fn try_from(id: u32) -> Result<Self, Self::Error> {
            if let Some(token) = Self::is::<GingerToken>(id) {
                return Ok(DynamicGingerSpace::Ginger(token));
            }
            if let Some(token) = Self::is::<MaoToken>(id) {
                return Ok(DynamicGingerSpace::Mao(token));
            }
            if let Some(token) = Self::is::<SingleToken>(id) {
                return Ok(DynamicGingerSpace::Single(token));
            }
            if let Some(token) = Self::is::<TextTokens>(id) {
                return Ok(DynamicGingerSpace::Text(token));
            }
            if let Some(offset) = Self::remainder(id) {
                return Ok(DynamicGingerSpace::Dynamic(offset));
            }
            // Since this has dynamic tokens, there's no real upper bound
            // We never actually return an error for DynamicGingerSpace
            unreachable!("DynamicGingerSpace accepts all values via dynamic tokens")
        }
    }

    #[test]
    fn test_accessing_tokens_in_space() {
        assert_eq!(GingerToken::TextStart.inside::<GingerSpace>(), 0);
        assert_eq!(MaoToken::ProgramStart.inside::<GingerSpace>(), 5);
        assert_eq!(SingleToken::Single.inside::<GingerSpace>(), 9);

        let mao_fn = MaoToken::Fn.inside::<GingerSpace>();
        assert_eq!(mao_fn, 7);

        let ginger_audio = GingerToken::AudioStart.inside::<GingerSpace>();
        assert_eq!(ginger_audio, 2);
    }

    #[test]
    fn test_is_token_in_space() {
        // Check if value 5 is a MaoToken (should be ProgramStart)
        assert_eq!(GingerSpace::is::<MaoToken>(5), Some(MaoToken::ProgramStart));
        assert_eq!(GingerSpace::is::<MaoToken>(6), Some(MaoToken::ProgramEnd));
        assert_eq!(GingerSpace::is::<MaoToken>(7), Some(MaoToken::Fn));
        assert_eq!(GingerSpace::is::<MaoToken>(8), Some(MaoToken::Struct));

        // Check if value 0 is a GingerToken (should be TextStart)
        assert_eq!(
            GingerSpace::is::<GingerToken>(0),
            Some(GingerToken::TextStart)
        );
        assert_eq!(
            GingerSpace::is::<GingerToken>(4),
            Some(GingerToken::AwaitAudio)
        );

        // Check SingleToken
        assert_eq!(GingerSpace::is::<SingleToken>(9), Some(SingleToken::Single));

        // Out of range tests
        assert!(GingerSpace::is::<MaoToken>(1000).is_none());
        assert!(GingerSpace::is::<MaoToken>(4).is_none()); // This is a GingerToken
        assert!(GingerSpace::is::<GingerToken>(5).is_none()); // This is a MaoToken
    }

    #[test]
    fn test_range_tokens() {
        // Test RangeToken::inside method
        assert_eq!(TextTokens::inside::<GingerSpace>(0), Some(10)); // First position
        assert_eq!(TextTokens::inside::<GingerSpace>(1), Some(11)); // Second position
        assert_eq!(TextTokens::inside::<GingerSpace>(999), Some(1009)); // Last position
        assert_eq!(TextTokens::inside::<GingerSpace>(1000), None); // Out of bounds
    }

    #[test]
    fn test_remainder_part() {
        assert_eq!(DynamicGingerSpace::remainder(1010), Some(0));
        assert_eq!(DynamicGingerSpace::remainder(1509), Some(499));
        assert_eq!(DynamicGingerSpace::remainder(500), None);

        // Static tokens still work
        assert_eq!(
            DynamicGingerSpace::is::<MaoToken>(5),
            Some(MaoToken::ProgramStart)
        );
    }

    #[test]
    fn test_is_with_range_tokens() {
        // Test is<T>() with RangeToken types (TextTokens)

        // Valid TextTokens in range [10, 1009]
        assert_eq!(GingerSpace::is::<TextTokens>(10), Some(TextTokens(0))); // First text token
        assert_eq!(GingerSpace::is::<TextTokens>(100), Some(TextTokens(90))); // Middle text token
        assert_eq!(GingerSpace::is::<TextTokens>(1009), Some(TextTokens(999))); // Last text token

        // Out of range - should return None
        assert_eq!(GingerSpace::is::<TextTokens>(9), None); // Before range
        assert_eq!(GingerSpace::is::<TextTokens>(1010), None); // After range
        assert_eq!(GingerSpace::is::<TextTokens>(0), None); // In GingerToken range
        assert_eq!(GingerSpace::is::<TextTokens>(5), None); // In MaoToken range

        // Test with DynamicGingerSpace too
        assert_eq!(
            DynamicGingerSpace::is::<TextTokens>(10),
            Some(TextTokens(0))
        );
        assert_eq!(
            DynamicGingerSpace::is::<TextTokens>(1009),
            Some(TextTokens(999))
        );
        assert_eq!(DynamicGingerSpace::is::<TextTokens>(1010), None); // In dynamic range
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
        assert_eq!(GingerSpace::is::<TextTokens>(10), Some(TextTokens(0))); // Global 10 -> Local 0
        assert_eq!(GingerSpace::is::<TextTokens>(50), Some(TextTokens(40))); // Global 50 -> Local 40
        assert_eq!(GingerSpace::is::<TextTokens>(500), Some(TextTokens(490))); // Global 500 -> Local 490
        assert_eq!(GingerSpace::is::<TextTokens>(1000), Some(TextTokens(990))); // Global 1000 -> Local 990

        // Test that local offset correctly maps to global position using inside()
        assert_eq!(TextTokens::inside::<GingerSpace>(0), Some(10)); // Local 0 -> Global 10
        assert_eq!(TextTokens::inside::<GingerSpace>(40), Some(50)); // Local 40 -> Global 50
        assert_eq!(TextTokens::inside::<GingerSpace>(490), Some(500)); // Local 490 -> Global 500 
        assert_eq!(TextTokens::inside::<GingerSpace>(990), Some(1000)); // Local 990 -> Global 1000

        // Test round-trip: global -> local -> global
        let global_pos = 250u32;
        if let Some(local_token) = GingerSpace::is::<TextTokens>(global_pos) {
            assert_eq!(local_token, TextTokens(240)); // 250 - 10 = 240
            if let Some(back_to_global) = TextTokens::inside::<GingerSpace>(local_token.0) {
                assert_eq!(back_to_global, global_pos); // Should get 250 back
            }
        }
    }

    #[test]
    fn test_offset_calculations() {
        // Same token should have same value in both spaces (since they have same static layout)
        let mao_token = MaoToken::ProgramStart;
        let value_space1 = mao_token.inside::<GingerSpace>();
        let value_space2 = mao_token.inside::<DynamicGingerSpace>();
        assert_eq!(value_space1, value_space2);

        // Test that is() works correctly for both spaces
        assert_eq!(GingerSpace::is::<MaoToken>(5), Some(MaoToken::ProgramStart));
        assert_eq!(
            DynamicGingerSpace::is::<MaoToken>(5),
            Some(MaoToken::ProgramStart)
        );
    }

    #[test]
    fn test_different_space_layouts() {
        use crate::space::{Position, TokenSpace};

        // Create a space with different offset layout using Space derive macro
        #[derive(Debug, PartialEq)]
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
        }

        impl TryFrom<u32> for AlternativeSpace {
            type Error = TokauError;

            fn try_from(id: u32) -> Result<Self, Self::Error> {
                if let Some(token) = Self::is::<MaoToken>(id) {
                    return Ok(AlternativeSpace::Mao(token));
                }
                if let Some(token) = Self::is::<SingleToken>(id) {
                    return Ok(AlternativeSpace::Single(token));
                }
                if let Some(token) = Self::is::<GingerToken>(id) {
                    return Ok(AlternativeSpace::Ginger(token));
                }
                if let Some(token) = Self::is::<TextTokens>(id) {
                    return Ok(AlternativeSpace::Text(token));
                }
                Err(TokauError::OutOfRange {
                    value: id,
                    max: Self::RESERVED,
                })
            }
        }

        // Test that same tokens have different values in different spaces
        let mao_in_dynamic = MaoToken::ProgramStart.inside::<DynamicGingerSpace>();
        let mao_in_alt = MaoToken::ProgramStart.inside::<AlternativeSpace>();

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
            .filter_map(|id| AlternativeSpace::is::<MaoToken>(id))
            .collect();
        // Only tokens 0,1 are present in our test vector, so we get ProgramStart, ProgramEnd
        assert_eq!(alt_maos, vec![MaoToken::ProgramStart, MaoToken::ProgramEnd]);

        // In DynamicGingerSpace, token 5 should be MaoToken::ProgramStart
        let dyn_maos: Vec<MaoToken> = tokens
            .into_iter()
            .filter_map(|id| DynamicGingerSpace::is::<MaoToken>(id))
            .collect();
        assert_eq!(
            dyn_maos,
            vec![MaoToken::ProgramStart, MaoToken::ProgramEnd, MaoToken::Fn]
        );
    }
}
