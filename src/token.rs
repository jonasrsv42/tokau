// Base trait for anything that can be positioned in a token space at compile time.
pub trait Token {
    const COUNT: u32;
    fn value(&self) -> u32;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::error::TokauError;

    #[derive(Debug, PartialEq, Clone, Copy)]
    #[repr(u32)]
    pub enum MaoToken {
        ProgramStart = 0,
        ProgramEnd = 1,
        Fn = 2,
        Struct = 3,
    }

    impl Token for MaoToken {
        const COUNT: u32 = 4;

        fn value(&self) -> u32 {
            *self as u32
        }
    }

    impl TryFrom<u32> for MaoToken {
        type Error = TokauError;

        fn try_from(value: u32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(MaoToken::ProgramStart),
                1 => Ok(MaoToken::ProgramEnd),
                2 => Ok(MaoToken::Fn),
                3 => Ok(MaoToken::Struct),
                _ => Err(TokauError::OutOfRange {
                    value,
                    max: Self::COUNT,
                }),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    #[repr(u32)]
    pub enum SingleToken {
        Single = 0,
    }

    impl Token for SingleToken {
        const COUNT: u32 = 1;

        fn value(&self) -> u32 {
            *self as u32
        }
    }

    impl TryFrom<u32> for SingleToken {
        type Error = TokauError;

        fn try_from(value: u32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(SingleToken::Single),
                _ => Err(TokauError::OutOfRange {
                    value,
                    max: Self::COUNT,
                }),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, Copy)]
    #[repr(u32)]
    pub enum GingerToken {
        TextStart = 0,
        TextEnd = 1,
        AudioStart = 2,
        AudioEnd = 3,
        AwaitAudio = 4,
    }

    impl Token for GingerToken {
        const COUNT: u32 = 5;

        fn value(&self) -> u32 {
            *self as u32
        }
    }

    impl TryFrom<u32> for GingerToken {
        type Error = TokauError;

        fn try_from(value: u32) -> Result<Self, Self::Error> {
            match value {
                0 => Ok(GingerToken::TextStart),
                1 => Ok(GingerToken::TextEnd),
                2 => Ok(GingerToken::AudioStart),
                3 => Ok(GingerToken::AudioEnd),
                4 => Ok(GingerToken::AwaitAudio),
                _ => Err(TokauError::OutOfRange {
                    value,
                    max: Self::COUNT,
                }),
            }
        }
    }

    // Example Range token
    #[derive(Debug, PartialEq, Clone, Copy)]
    pub struct TextTokens(pub u32);

    impl Token for TextTokens {
        const COUNT: u32 = 1000; // 1000 text tokens

        fn value(&self) -> u32 {
            self.0
        }
    }

    impl TryFrom<u32> for TextTokens {
        type Error = TokauError;

        fn try_from(offset: u32) -> Result<Self, Self::Error> {
            if offset < Self::COUNT {
                Ok(TextTokens(offset))
            } else {
                Err(TokauError::OutOfRange {
                    value: offset,
                    max: Self::COUNT,
                })
            }
        }
    }

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
        assert_eq!(
            MaoToken::try_from(4),
            Err(TokauError::OutOfRange { value: 4, max: 4 })
        );

        // Test SingleToken conversions
        assert_eq!(SingleToken::try_from(0), Ok(SingleToken::Single));
        assert_eq!(
            SingleToken::try_from(1),
            Err(TokauError::OutOfRange { value: 1, max: 1 })
        );

        // Test GingerToken conversions
        assert_eq!(GingerToken::try_from(0), Ok(GingerToken::TextStart));
        assert_eq!(GingerToken::try_from(4), Ok(GingerToken::AwaitAudio));
        assert_eq!(
            GingerToken::try_from(5),
            Err(TokauError::OutOfRange { value: 5, max: 5 })
        );
    }
}
