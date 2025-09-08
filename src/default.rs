use crate::space::{Position, TokenSpace};
use crate::token::{NameToken, RangeToken};

// Create separate types to avoid conflicting implementations
#[derive(Debug, PartialEq)]
pub enum NameTokenSpace<T: NameToken> {
    Token(T),
    Dynamic(u32),
}

#[derive(Debug, PartialEq)]
pub enum RangeTokenSpace<T: RangeToken> {
    Token(T),
    Dynamic(u32),
}

impl<T: NameToken> Position<T> for NameTokenSpace<T> {
    const OFFSET: u32 = 0;
}

impl<T: RangeToken> Position<T> for RangeTokenSpace<T> {
    const OFFSET: u32 = 0;
}

impl<T> TokenSpace for NameTokenSpace<T>
where
    T: NameToken + TryFrom<u32>,
{
    const RESERVED: u32 = T::COUNT;
}

impl<T> TryFrom<u32> for NameTokenSpace<T>
where
    T: NameToken + TryFrom<u32>,
{
    type Error = ();

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if let Some(token) = Self::is::<T>(id) {
            return Ok(NameTokenSpace::Token(token));
        }
        if let Some(offset) = Self::remainder(id) {
            return Ok(NameTokenSpace::Dynamic(offset));
        }
        Err(())
    }
}

impl<T> TokenSpace for RangeTokenSpace<T>
where
    T: RangeToken + TryFrom<u32>,
{
    const RESERVED: u32 = T::COUNT;
}

impl<T> TryFrom<u32> for RangeTokenSpace<T>
where
    T: RangeToken + TryFrom<u32>,
{
    type Error = ();

    fn try_from(id: u32) -> Result<Self, Self::Error> {
        if let Some(token) = Self::is::<T>(id) {
            return Ok(RangeTokenSpace::Token(token));
        }
        if let Some(offset) = Self::remainder(id) {
            return Ok(RangeTokenSpace::Dynamic(offset));
        }
        Err(())
    }
}

// Remove Default implementation - doesn't make sense for enum without knowing which variant to use

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ext::TokenFilter;
    use crate::token::NameToken;
    use crate::token::tests::*;

    #[test]
    fn test_default_space() {
        // Test NameTokenSpace with NameToken tokens
        let mao_start = MaoToken::ProgramStart.inside::<NameTokenSpace<MaoToken>>();
        assert_eq!(mao_start, 0); // Should be at offset 0

        let mao_fn = MaoToken::Fn.inside::<NameTokenSpace<MaoToken>>();
        assert_eq!(mao_fn, 2); // Direct value mapping

        // Test is() with NameTokenSpace
        assert_eq!(
            NameTokenSpace::<MaoToken>::is::<MaoToken>(0),
            Some(MaoToken::ProgramStart)
        );
        assert_eq!(
            NameTokenSpace::<MaoToken>::is::<MaoToken>(3),
            Some(MaoToken::Struct)
        );
        assert_eq!(NameTokenSpace::<MaoToken>::is::<MaoToken>(4), None); // Out of range

        // Test with GingerToken
        let ginger_audio = GingerToken::AudioStart.inside::<NameTokenSpace<GingerToken>>();
        assert_eq!(ginger_audio, 2); // Direct value mapping

        // Test filtering with NameTokenSpace
        let tokens = vec![0, 1, 2, 3, 4, 5];
        let mao_tokens: Vec<MaoToken> = tokens
            .clone()
            .into_iter()
            .is::<NameTokenSpace<MaoToken>, MaoToken>()
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
        assert_eq!(NameTokenSpace::<MaoToken>::remainder(0), None); // In static range
        assert_eq!(NameTokenSpace::<MaoToken>::remainder(4), Some(0)); // First dynamic position
        assert_eq!(NameTokenSpace::<MaoToken>::remainder(100), Some(96)); // Dynamic position 96
    }

    #[test]
    fn test_default_space_with_dynamic_tokens() {
        // Test static tokens still work
        let mao_start = MaoToken::ProgramStart.inside::<NameTokenSpace<MaoToken>>();
        assert_eq!(mao_start, 0);
        assert_eq!(
            NameTokenSpace::<MaoToken>::is::<MaoToken>(0),
            Some(MaoToken::ProgramStart)
        );

        // Test dynamic tokens (no bounds checking now)
        assert_eq!(NameTokenSpace::<MaoToken>::remainder(4), Some(0)); // First dynamic token
        assert_eq!(NameTokenSpace::<MaoToken>::remainder(103), Some(99)); // Dynamic token at offset 99
        assert_eq!(NameTokenSpace::<MaoToken>::remainder(2), None); // In static range, not dynamic

        // Test filtering with dynamic tokens
        let tokens = vec![0, 1, 2, 3, 4, 5, 50, 103, 104, 200];

        // Filter static tokens
        let mao_tokens: Vec<MaoToken> = tokens
            .clone()
            .into_iter()
            .is::<NameTokenSpace<MaoToken>, MaoToken>()
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

        // Filter dynamic tokens (no longer bounded by count)
        let dynamic_tokens: Vec<u32> = tokens
            .into_iter()
            .remainders::<NameTokenSpace<MaoToken>>()
            .collect();
        assert_eq!(dynamic_tokens, vec![4, 5, 50, 103, 104, 200]); // All tokens >= RESERVED
    }
}
