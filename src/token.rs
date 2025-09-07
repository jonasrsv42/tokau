use crate::space::Position;
// Base trait for anything that can be positioned in a token space at compile time.
pub trait Token {
    const COUNT: u32;
}

// For discrete/reserved tokens with specific values and instances
pub trait NameToken: Token + Sized {
    fn value(&self) -> u32;

    fn inside<S: Position<Self>>(&self) -> u32 {
        S::value(self)
    }
}

// For range tokens without specific instances - just represents a contiguous range
pub trait RangeToken: Token {
    // No instances, just represents COUNT tokens as a range
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum MaoToken {
        ProgramStart,
        ProgramEnd,
        Fn,
        Struct,
    }

    impl Token for MaoToken {
        const COUNT: u32 = 4;
    }

    impl NameToken for MaoToken {
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

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum SingleToken {
        Single,
    }

    impl Token for SingleToken {
        const COUNT: u32 = 1;
    }

    impl NameToken for SingleToken {
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

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum GingerToken {
        TextStart,
        TextEnd,
        AudioStart,
        AudioEnd,
        AwaitAudio,
    }

    impl Token for GingerToken {
        const COUNT: u32 = 5;
    }

    impl NameToken for GingerToken {
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

    // Example Range token
    pub struct TextTokens;

    impl Token for TextTokens {
        const COUNT: u32 = 1000; // 1000 text tokens
    }

    impl RangeToken for TextTokens {}

    #[test]
    fn test_token_counts() {
        assert_eq!(MaoToken::COUNT, 4);
        assert_eq!(SingleToken::COUNT, 1);
        assert_eq!(GingerToken::COUNT, 5);
        assert_eq!(TextTokens::COUNT, 1000);
    }

    #[test]
    fn test_special_token_values() {
        assert_eq!(MaoToken::ProgramStart.value(), 0);
        assert_eq!(MaoToken::ProgramEnd.value(), 1);
        assert_eq!(MaoToken::Fn.value(), 2);
        assert_eq!(MaoToken::Struct.value(), 3);

        assert_eq!(SingleToken::Single.value(), 0);

        assert_eq!(GingerToken::TextStart.value(), 0);
        assert_eq!(GingerToken::TextEnd.value(), 1);
        assert_eq!(GingerToken::AudioStart.value(), 2);
        assert_eq!(GingerToken::AudioEnd.value(), 3);
        assert_eq!(GingerToken::AwaitAudio.value(), 4);
    }

    #[test]
    fn test_try_from_conversions() {
        // Test MaoToken conversions
        assert_eq!(MaoToken::try_from(0), Ok(MaoToken::ProgramStart));
        assert_eq!(MaoToken::try_from(1), Ok(MaoToken::ProgramEnd));
        assert_eq!(MaoToken::try_from(2), Ok(MaoToken::Fn));
        assert_eq!(MaoToken::try_from(3), Ok(MaoToken::Struct));
        assert_eq!(MaoToken::try_from(4), Err(()));

        // Test SingleToken conversions
        assert_eq!(SingleToken::try_from(0), Ok(SingleToken::Single));
        assert_eq!(SingleToken::try_from(1), Err(()));

        // Test GingerToken conversions
        assert_eq!(GingerToken::try_from(0), Ok(GingerToken::TextStart));
        assert_eq!(GingerToken::try_from(4), Ok(GingerToken::AwaitAudio));
        assert_eq!(GingerToken::try_from(5), Err(()));
    }
}
