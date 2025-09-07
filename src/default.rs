use crate::space::{Position, TokenSpace};
use crate::token::Token;
use std::marker::PhantomData;

// Default space for single token types
pub struct DefaultSpace<T: Token> {
    _phantom: PhantomData<T>,
}

impl<T: Token> Position<T> for DefaultSpace<T> {
    const OFFSET: u32 = 0;
}

impl<T: Token> TokenSpace for DefaultSpace<T> {
    const RESERVED: u32 = T::COUNT;

    fn count(&self) -> u32 {
        Self::RESERVED
    }
}

impl<T: Token> Default for DefaultSpace<T> {
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ext::TokenFilter;
    use crate::token::Special;
    use crate::token::tests::*;

    #[test]
    fn test_default_space() {
        // Test DefaultSpace with Special tokens
        let mao_start = MaoToken::ProgramStart.in_::<DefaultSpace<MaoToken>>();
        assert_eq!(mao_start, 0); // Should be at offset 0

        let mao_fn = MaoToken::Fn.in_::<DefaultSpace<MaoToken>>();
        assert_eq!(mao_fn, 2); // Direct value mapping

        // Test is() with DefaultSpace
        assert_eq!(
            DefaultSpace::<MaoToken>::is::<MaoToken>(0),
            Some(MaoToken::ProgramStart)
        );
        assert_eq!(
            DefaultSpace::<MaoToken>::is::<MaoToken>(3),
            Some(MaoToken::Struct)
        );
        assert_eq!(DefaultSpace::<MaoToken>::is::<MaoToken>(4), None); // Out of range

        // Test with GingerToken
        let ginger_audio = GingerToken::AudioStart.in_::<DefaultSpace<GingerToken>>();
        assert_eq!(ginger_audio, 2); // Direct value mapping

        // Test with Range tokens
        assert_eq!(DefaultSpace::<TextTokens>::to::<TextTokens>(0), Some(0));
        assert_eq!(DefaultSpace::<TextTokens>::to::<TextTokens>(999), Some(999));
        assert_eq!(DefaultSpace::<TextTokens>::to::<TextTokens>(1000), None); // Out of range

        // Test that to() now works with Special tokens too
        assert_eq!(DefaultSpace::<MaoToken>::to::<MaoToken>(0), Some(0));
        assert_eq!(DefaultSpace::<MaoToken>::to::<MaoToken>(2), Some(2));
        assert_eq!(DefaultSpace::<MaoToken>::to::<MaoToken>(4), None); // Out of range

        // Test filtering with DefaultSpace
        let tokens = vec![0, 1, 2, 3, 4, 5];
        let mao_tokens: Vec<MaoToken> = tokens
            .clone()
            .into_iter()
            .specials::<DefaultSpace<MaoToken>, MaoToken>()
            .collect();
        assert_eq!(
            mao_tokens,
            vec![
                MaoToken::ProgramStart,
                MaoToken::ProgramEnd,
                MaoToken::Fn,
                MaoToken::Struct,
            ]
        );

        // Test that ranges() now works with Special tokens
        let mao_ranges: Vec<u32> = tokens
            .clone()
            .into_iter()
            .ranges::<DefaultSpace<MaoToken>, MaoToken>()
            .collect();
        assert_eq!(mao_ranges, vec![0, 1, 2, 3]); // All valid MaoToken IDs

        // Test that DefaultSpace has no dynamic part (only static tokens)
        let space = DefaultSpace::<MaoToken>::default();
        assert_eq!(space.dynamic(0), None);
        assert_eq!(space.dynamic(100), None);
    }
}
