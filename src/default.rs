use crate::error::TokauError;
use crate::space::{Position, TokenSpace};
use crate::token::Token;

// Create separate types to avoid conflicting implementations
#[derive(Debug, PartialEq)]
pub enum DefaultTokenSpace<T: Token> {
    Token(T),
    Dynamic(u32),
}

impl<T: Token> Position<T> for DefaultTokenSpace<T> {
    const OFFSET: u32 = 0;
}

impl<T> TokenSpace for DefaultTokenSpace<T>
where
    T: Token + TryFrom<u32, Error = TokauError>,
{
    const RESERVED: u32 = T::COUNT;
}

impl<T> TryFrom<u32> for DefaultTokenSpace<T>
where
    T: Token + TryFrom<u32, Error = TokauError>,
{
    type Error = TokauError;

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if let Some(token) = Self::try_as::<T>(id) {
            return Ok(DefaultTokenSpace::Token(token));
        }
        if let Some(offset) = Self::remainder(id) {
            return Ok(DefaultTokenSpace::Dynamic(offset));
        }
        // Since this has dynamic tokens, it accepts all values
        unreachable!("NameTokenSpace with dynamic tokens accepts all values")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ext::TokenFilter;
    use crate::token::tests::*;

    #[test]
    fn test_default_space() {
        // Test NameTokenSpace with NameToken tokens
        let mao_start = DefaultTokenSpace::<MaoToken>::position_of(MaoToken::ProgramStart);
        assert_eq!(mao_start, 0); // Should be at offset 0

        let mao_fn = DefaultTokenSpace::<MaoToken>::position_of(MaoToken::Fn);
        assert_eq!(mao_fn, 2); // Direct value mapping

        // Test try_as() with NameTokenSpace
        assert_eq!(
            DefaultTokenSpace::<MaoToken>::try_as::<MaoToken>(0),
            Some(MaoToken::ProgramStart)
        );
        assert_eq!(
            DefaultTokenSpace::<MaoToken>::try_as::<MaoToken>(3),
            Some(MaoToken::Struct)
        );
        assert_eq!(DefaultTokenSpace::<MaoToken>::try_as::<MaoToken>(4), None); // Out of range

        // Test with GingerToken
        let ginger_audio = DefaultTokenSpace::<GingerToken>::position_of(GingerToken::AudioStart);
        assert_eq!(ginger_audio, 2); // Direct value mapping

        // Test filtering with NameTokenSpace
        let tokens = vec![0, 1, 2, 3, 4, 5];
        let mao_tokens: Vec<MaoToken> = tokens
            .clone()
            .into_iter()
            .try_as::<DefaultTokenSpace<MaoToken>, MaoToken>()
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

        // Test that NameTokenSpace dynamic method works without bounds
        assert_eq!(DefaultTokenSpace::<MaoToken>::remainder(0), None); // In static range
        assert_eq!(DefaultTokenSpace::<MaoToken>::remainder(4), Some(0)); // First dynamic position
        assert_eq!(DefaultTokenSpace::<MaoToken>::remainder(100), Some(96)); // Dynamic position 96
    }

    #[test]
    fn test_default_space_with_dynamic_tokens() {
        // Test static tokens still work
        let mao_start = DefaultTokenSpace::<MaoToken>::position_of(MaoToken::ProgramStart);
        assert_eq!(mao_start, 0);
        assert_eq!(
            DefaultTokenSpace::<MaoToken>::try_as::<MaoToken>(0),
            Some(MaoToken::ProgramStart)
        );

        // Test dynamic tokens (no bounds checking now)
        assert_eq!(DefaultTokenSpace::<MaoToken>::remainder(4), Some(0)); // First dynamic token
        assert_eq!(DefaultTokenSpace::<MaoToken>::remainder(103), Some(99)); // Dynamic token at offset 99
        assert_eq!(DefaultTokenSpace::<MaoToken>::remainder(2), None); // In static range, not dynamic

        // Test filtering with dynamic tokens
        let tokens = vec![0, 1, 2, 3, 4, 5, 50, 103, 104, 200];

        // Filter static tokens
        let mao_tokens: Vec<MaoToken> = tokens
            .clone()
            .into_iter()
            .try_as::<DefaultTokenSpace<MaoToken>, MaoToken>()
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

        // Filter remainder values of dynamic tokens
        let remainder_values: Vec<u32> = tokens
            .into_iter()
            .remainders::<DefaultTokenSpace<MaoToken>>()
            .collect();
        assert_eq!(remainder_values, vec![0, 1, 46, 99, 100, 196]); // Remainder values (token_id - RESERVED)
    }
}
