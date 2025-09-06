// Base trait for anything that can be positioned in a token space
pub trait Token {
    const COUNT: u32;
}

// For discrete/reserved tokens with specific values and instances
pub trait Special: Token + Sized {
    fn value(&self) -> u32;
    
    fn in_<S: Position<Self>>(&self) -> u32 {
        S::value(self)
    }
}

// For range tokens without specific instances - just represents a contiguous range
pub trait Range: Token {
    // No instances, just represents COUNT tokens as a range
}

pub trait Position<TokenType: Token> {
    const OFFSET: u32;

    // For Special tokens - convert instance to global value
    fn value(token: &TokenType) -> u32 
    where TokenType: Special 
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
        T: TryFrom<u32>
    {
        let start = <Self as Position<T>>::OFFSET;
        if value >= start && value < start + T::COUNT {
            T::try_from(value - start).ok()
        } else {
            None
        }
    }
    
    // For Range tokens - check if value is in range and return offset
    fn to<T: Range>(value: u32) -> Option<u32>
    where 
        Self: Position<T>
    {
        let start = <Self as Position<T>>::OFFSET;
        if value >= start && value < start + T::COUNT {
            Some(value - start) // Return offset within the range
        } else {
            None
        }
    }
    
    // For dynamic tail - check if value is in dynamic part and return tail offset
    fn tail(&self, value: u32) -> Option<u32> {
        if value >= Self::RESERVED && value < self.count() {
            Some(value - Self::RESERVED)
        } else {
            None
        }
    }
}

// Extension trait for filtering iterables by token type
pub trait TokenFilter: Iterator<Item = u32> + Sized {
    fn tails<S: TokenSpace>(self, space: &S) -> impl Iterator<Item = u32> {
        self.filter_map(move |id| space.tail(id).map(|_| id))
    }
    
    fn specials<S: TokenSpace, T: Special>(self) -> impl Iterator<Item = T> 
    where 
        S: Position<T>,
        T: TryFrom<u32>
    {
        self.filter_map(|id| S::is::<T>(id))
    }
    
    fn ranges<S: TokenSpace, T: Range>(self) -> impl Iterator<Item = u32>
    where
        S: Position<T>
    {
        self.filter_map(|id| S::to::<T>(id).map(|_| id))
    }
}

// Implementation for all iterators over u32
impl<I: Iterator<Item = u32> + Sized> TokenFilter for I {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    enum MaoToken {
        ProgramStart,
        ProgramEnd,
        Fn,
        Struct,
    }

    impl Token for MaoToken {
        const COUNT: u32 = 4;
    }
    
    impl Special for MaoToken {
        fn value(&self) -> u32 {
            match self {
                MaoToken::ProgramStart => 0,
                MaoToken::ProgramEnd => 1,
                MaoToken::Fn => 2,
                MaoToken::Struct => 3,
            }
        }
    }
    
    impl TryFrom<u32> for MaoToken {
        type Error = ();
        
        fn try_from(value: u32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(MaoToken::ProgramStart),
                1 => Ok(MaoToken::ProgramEnd),
                2 => Ok(MaoToken::Fn),
                3 => Ok(MaoToken::Struct),
                _ => Err(()),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    enum SingleToken {
        Single,
    }

    impl Token for SingleToken {
        const COUNT: u32 = 1;
    }
    
    impl Special for SingleToken {
        fn value(&self) -> u32 {
            0
        }
    }
    
    impl TryFrom<u32> for SingleToken {
        type Error = ();
        
        fn try_from(value: u32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(SingleToken::Single),
                _ => Err(()),
            }
        }
    }

    #[derive(Debug, PartialEq)]
    enum GingerToken {
        TextStart,
        TextEnd,
        AudioStart,
        AudioEnd,
        AwaitAudio,
    }

    impl Token for GingerToken {
        const COUNT: u32 = 5;
    }
    
    impl Special for GingerToken {
        fn value(&self) -> u32 {
            match self {
                GingerToken::TextStart => 0,
                GingerToken::TextEnd => 1,
                GingerToken::AudioStart => 2,
                GingerToken::AudioEnd => 3,
                GingerToken::AwaitAudio => 4,
            }
        }
    }
    
    impl TryFrom<u32> for GingerToken {
        type Error = ();
        
        fn try_from(value: u32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(GingerToken::TextStart),
                1 => Ok(GingerToken::TextEnd),
                2 => Ok(GingerToken::AudioStart),
                3 => Ok(GingerToken::AudioEnd),
                4 => Ok(GingerToken::AwaitAudio),
                _ => Err(()),
            }
        }
    }

    struct GingerSpace {}

    impl Position<MaoToken> for GingerSpace {
        const OFFSET: u32 = GingerToken::COUNT;
    }

    impl Position<GingerToken> for GingerSpace {
        const OFFSET: u32 = 0;
    }

    impl Position<SingleToken> for GingerSpace {
        const OFFSET: u32 = GingerToken::COUNT + MaoToken::COUNT;
    }
    
    // Example Range token
    struct TextTokens;
    
    impl Token for TextTokens {
        const COUNT: u32 = 1000; // 1000 text tokens
    }
    
    impl Range for TextTokens {}
    
    impl Position<TextTokens> for GingerSpace {
        const OFFSET: u32 = GingerToken::COUNT + MaoToken::COUNT + SingleToken::COUNT;
    }

    impl TokenSpace for GingerSpace {
        const RESERVED: u32 = GingerToken::COUNT + MaoToken::COUNT + SingleToken::COUNT + TextTokens::COUNT;
        
        fn count(&self) -> u32 {
            Self::RESERVED // For now, no dynamic tail
        }
    }

    #[test]
    fn test_accessing_tokens_in_space() {
        // Much cleaner syntax with in_!
        assert_eq!(GingerToken::TextStart.in_::<GingerSpace>(), 0);
        assert_eq!(MaoToken::ProgramStart.in_::<GingerSpace>(), 5);
        assert_eq!(SingleToken::Single.in_::<GingerSpace>(), 9);
        
        // Can also store in variables
        let mao_fn = MaoToken::Fn.in_::<GingerSpace>();
        assert_eq!(mao_fn, 7);
        
        let ginger_audio = GingerToken::AudioStart.in_::<GingerSpace>();
        assert_eq!(ginger_audio, 2);
    }
    
    #[test]
    fn test_is_token_in_space() {
        // Test GingerSpace::is with different token types
        
        // Check if value 5 is a MaoToken (should be ProgramStart)
        assert_eq!(GingerSpace::is::<MaoToken>(5), Some(MaoToken::ProgramStart));
        assert_eq!(GingerSpace::is::<MaoToken>(6), Some(MaoToken::ProgramEnd));
        assert_eq!(GingerSpace::is::<MaoToken>(7), Some(MaoToken::Fn));
        assert_eq!(GingerSpace::is::<MaoToken>(8), Some(MaoToken::Struct));
        
        // Check if value 0 is a GingerToken (should be TextStart)
        assert_eq!(GingerSpace::is::<GingerToken>(0), Some(GingerToken::TextStart));
        assert_eq!(GingerSpace::is::<GingerToken>(4), Some(GingerToken::AwaitAudio));
        
        // Check SingleToken
        assert_eq!(GingerSpace::is::<SingleToken>(9), Some(SingleToken::Single));
        
        // Out of range tests
        assert!(GingerSpace::is::<MaoToken>(1000).is_none());
        assert!(GingerSpace::is::<MaoToken>(4).is_none()); // This is a GingerToken
        assert!(GingerSpace::is::<GingerToken>(5).is_none()); // This is a MaoToken
    }
    
    #[test]
    fn test_range_tokens() {
        // Test Range tokens with to()
        // TextTokens start at offset 10 (5 + 4 + 1)
        assert_eq!(GingerSpace::to::<TextTokens>(10), Some(0));  // First text token
        assert_eq!(GingerSpace::to::<TextTokens>(11), Some(1));  // Second text token
        assert_eq!(GingerSpace::to::<TextTokens>(1009), Some(999)); // Last text token
        
        // Out of range
        assert!(GingerSpace::to::<TextTokens>(9).is_none());   // Before range
        assert!(GingerSpace::to::<TextTokens>(1010).is_none()); // After range
        
        // Other token types shouldn't match as ranges
        assert!(GingerSpace::to::<TextTokens>(5).is_none()); // MaoToken area
    }
    
    // Example of a dynamic token space
    struct DynamicGingerSpace {
        vocab_size: u32,
    }
    
    impl DynamicGingerSpace {
        fn new(vocab_size: u32) -> Self {
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
        const RESERVED: u32 = GingerToken::COUNT + MaoToken::COUNT + SingleToken::COUNT + TextTokens::COUNT;
        
        fn count(&self) -> u32 {
            Self::RESERVED + self.vocab_size // Include dynamic vocabulary
        }
    }
    
    #[test]
    fn test_dynamic_tail() {
        let space = DynamicGingerSpace::new(500); // 500 dynamic vocab tokens
        
        // Check total count
        assert_eq!(space.count(), 1010 + 500); // RESERVED + vocab_size
        
        // Test tail function
        assert_eq!(space.tail(1010), Some(0));   // First dynamic token
        assert_eq!(space.tail(1509), Some(499)); // Last dynamic token
        assert_eq!(space.tail(1510), None);      // Beyond range
        assert_eq!(space.tail(500), None);       // In static range, not tail
        
        // Static tokens still work
        assert_eq!(DynamicGingerSpace::is::<MaoToken>(5), Some(MaoToken::ProgramStart));
        assert_eq!(DynamicGingerSpace::to::<TextTokens>(1009), Some(999));
    }
    
    #[test]
    fn test_token_filter_extension() {
        let space = DynamicGingerSpace::new(500);
        let tokens: Vec<u32> = vec![0, 5, 6, 7, 10, 50, 1010, 1011, 1200, 1600];
        
        // Filter to only tail tokens
        let tail_tokens: Vec<u32> = tokens.clone().into_iter().tails(&space).collect();
        assert_eq!(tail_tokens, vec![1010, 1011, 1200]);
        
        // Filter to only MaoTokens (returns actual token instances)
        let mao_tokens: Vec<MaoToken> = tokens.clone().into_iter()
            .specials::<DynamicGingerSpace, MaoToken>()
            .collect();
        assert_eq!(mao_tokens, vec![
            MaoToken::ProgramStart, // from token 5
            MaoToken::ProgramEnd,   // from token 6
            MaoToken::Fn,           // from token 7
        ]);
        
        // Filter to only TextTokens (returns token IDs)
        let text_tokens: Vec<u32> = tokens.into_iter()
            .ranges::<DynamicGingerSpace, TextTokens>()
            .collect();
        assert_eq!(text_tokens, vec![10, 50]);
    }

}
