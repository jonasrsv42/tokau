use crate::token::{Special, Token};

pub trait Position<TokenType: Token> {
    const OFFSET: u32;

    // For Special tokens - convert instance to global value
    fn value(token: &TokenType) -> u32
    where
        TokenType: Special,
    {
        token.value() + Self::OFFSET
    }
}

pub trait TokenSpace: Sized {
    const RESERVED: u32; // Fixed/static part of the token space

    fn count(&self) -> u32; // Total count including dynamic parts

    // For Special tokens - try to convert global value back to token instance
    fn is<T: Special>(value: u32) -> Option<T>
    where
        Self: Position<T>,
        T: TryFrom<u32>,
    {
        let start = <Self as Position<T>>::OFFSET;
        if value >= start && value < start + T::COUNT {
            T::try_from(value - start).ok()
        } else {
            None
        }
    }

    // Check if value is in token range and return offset
    fn to<T: Token>(value: u32) -> Option<u32>
    where
        Self: Position<T>,
    {
        let start = <Self as Position<T>>::OFFSET;
        if value >= start && value < start + T::COUNT {
            Some(value - start) // Return offset within the range
        } else {
            None
        }
    }

    // For dynamic part - check if value is in dynamic part and return offset
    fn dynamic(&self, value: u32) -> Option<u32> {
        if value >= Self::RESERVED && value < self.count() {
            Some(value - Self::RESERVED)
        } else {
            None
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::token::Special;
    use crate::token::tests::*;

    pub(crate) struct GingerSpace {}

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

    impl TokenSpace for GingerSpace {
        const RESERVED: u32 =
            GingerToken::COUNT + MaoToken::COUNT + SingleToken::COUNT + TextTokens::COUNT;

        fn count(&self) -> u32 {
            Self::RESERVED // For now, no dynamic part
        }
    }

    // Example of a dynamic token space
    pub(crate) struct DynamicGingerSpace {
        vocab_size: u32,
    }

    impl DynamicGingerSpace {
        pub(crate) fn new(vocab_size: u32) -> Self {
            Self { vocab_size }
        }
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

        fn count(&self) -> u32 {
            Self::RESERVED + self.vocab_size // Include dynamic vocabulary
        }
    }

    #[test]
    fn test_accessing_tokens_in_space() {
        assert_eq!(GingerToken::TextStart.in_::<GingerSpace>(), 0);
        assert_eq!(MaoToken::ProgramStart.in_::<GingerSpace>(), 5);
        assert_eq!(SingleToken::Single.in_::<GingerSpace>(), 9);

        let mao_fn = MaoToken::Fn.in_::<GingerSpace>();
        assert_eq!(mao_fn, 7);

        let ginger_audio = GingerToken::AudioStart.in_::<GingerSpace>();
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
        // TextTokens start at offset 10 (5 + 4 + 1)
        assert_eq!(GingerSpace::to::<TextTokens>(10), Some(0)); // First text token
        assert_eq!(GingerSpace::to::<TextTokens>(11), Some(1)); // Second text token
        assert_eq!(GingerSpace::to::<TextTokens>(1009), Some(999)); // Last text token

        // Out of range
        assert!(GingerSpace::to::<TextTokens>(9).is_none()); // Before range
        assert!(GingerSpace::to::<TextTokens>(1010).is_none()); // After range

        // Other token types shouldn't match as ranges
        assert!(GingerSpace::to::<TextTokens>(5).is_none()); // MaoToken area
    }

    #[test]
    fn test_dynamic_part() {
        let space = DynamicGingerSpace::new(500); // 500 dynamic vocab tokens

        // Check total count
        assert_eq!(space.count(), 1010 + 500); // RESERVED + vocab_size

        // Test dynamic function
        assert_eq!(space.dynamic(1010), Some(0)); // First dynamic token
        assert_eq!(space.dynamic(1509), Some(499)); // Last dynamic token
        assert_eq!(space.dynamic(1510), None); // Beyond range
        assert_eq!(space.dynamic(500), None); // In static range, not dynamic

        // Static tokens still work
        assert_eq!(
            DynamicGingerSpace::is::<MaoToken>(5),
            Some(MaoToken::ProgramStart)
        );
        assert_eq!(DynamicGingerSpace::to::<TextTokens>(1009), Some(999));
    }

    #[test]
    fn test_offset_calculations() {
        // Same token should have same value in both spaces (since they have same static layout)
        let mao_token = MaoToken::ProgramStart;
        let value_space1 = mao_token.in_::<GingerSpace>();
        let value_space2 = mao_token.in_::<DynamicGingerSpace>();
        assert_eq!(value_space1, value_space2);

        // Test that is() works correctly for both spaces
        assert_eq!(GingerSpace::is::<MaoToken>(5), Some(MaoToken::ProgramStart));
        assert_eq!(
            DynamicGingerSpace::is::<MaoToken>(5),
            Some(MaoToken::ProgramStart)
        );

        // Test range boundaries are consistent
        assert_eq!(GingerSpace::to::<TextTokens>(10), Some(0)); // First text token
        assert_eq!(DynamicGingerSpace::to::<TextTokens>(10), Some(0)); // Should be same

        assert_eq!(GingerSpace::to::<TextTokens>(1009), Some(999)); // Last text token  
        assert_eq!(DynamicGingerSpace::to::<TextTokens>(1009), Some(999)); // Should be same
    }

    #[test]
    fn test_different_space_layouts() {
        // Create a space with different offset layout
        struct AlternativeSpace;

        impl TokenSpace for AlternativeSpace {
            const RESERVED: u32 =
                MaoToken::COUNT + SingleToken::COUNT + GingerToken::COUNT + TextTokens::COUNT;

            fn count(&self) -> u32 {
                Self::RESERVED
            }
        }

        // Different offset order: Mao first, Single second, Ginger third, Text last
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

        // Test that same tokens have different values in different spaces
        let mao_in_dynamic = MaoToken::ProgramStart.in_::<DynamicGingerSpace>();
        let mao_in_alt = MaoToken::ProgramStart.in_::<AlternativeSpace>();

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
