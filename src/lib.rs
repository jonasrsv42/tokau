pub trait Token {
    fn value(&self) -> u32;

    const COUNT: u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    enum MaoToken {
        ProgramStart,
        ProgramEnd,
        Fn,
        Struct,
    }

    impl Token for MaoToken {
        fn value(&self) -> u32 {
            match self {
                MaoToken::ProgramStart => 0,
                MaoToken::ProgramEnd => 1,
                MaoToken::Fn => 2,
                MaoToken::Struct => 3,
            }
        }

        const COUNT: u32 = 4;
    }

    enum SingleToken {
        Single,
    }

    impl Token for SingleToken {
        fn value(&self) -> u32 {
            0
        }

        const COUNT: u32 = 1;
    }

    enum GingerToken {
        Mao(MaoToken),
        TextStart,
        TextEnd,
        AudioStart,
        AudioEnd,
        AwaitAudio,
        Single(SingleToken),
    }

    impl Token for GingerToken {
        fn value(&self) -> u32 {
            match self {
                GingerToken::TextStart =>  0,
                GingerToken::TextEnd =>  1,
                GingerToken::AudioStart =>  2,
                GingerToken::AudioEnd =>  3,
                GingerToken::AwaitAudio =>  4,
                GingerToken::Mao(mao) => 5 + mao.value(),
                GingerToken::Single(single) => 5 + MaoToken::COUNT + single.value(),
            }
        }

        const COUNT: u32 = 5 + MaoToken::COUNT + SingleToken::COUNT;
    }

    impl From<MaoToken> for GingerToken {
        fn from(token: MaoToken) -> Self {
            GingerToken::Mao(token)
        }
    }

    impl From<SingleToken> for GingerToken {
        fn from(token: SingleToken) -> Self {
            GingerToken::Single(token)
        }
    }

    #[test]
    fn test_token_local_values() {
        // Test MaoToken values
        assert_eq!(MaoToken::ProgramStart.value(), 0);
        assert_eq!(MaoToken::ProgramEnd.value(), 1);
        assert_eq!(MaoToken::Fn.value(), 2);
        assert_eq!(MaoToken::Struct.value(), 3);

        // Test SingleToken value
        assert_eq!(SingleToken::Single.value(), 0);
    }

    #[test]
    fn test_ginger_token_composed_values() {
        // Test native GingerToken values - start at 0
        assert_eq!(GingerToken::TextStart.value(), 0);
        assert_eq!(GingerToken::TextEnd.value(), 1);
        assert_eq!(GingerToken::AudioStart.value(), 2);
        assert_eq!(GingerToken::AudioEnd.value(), 3);
        assert_eq!(GingerToken::AwaitAudio.value(), 4);

        // Test wrapped MaoToken values - offset by 5
        assert_eq!(GingerToken::Mao(MaoToken::ProgramStart).value(), 5);
        assert_eq!(GingerToken::Mao(MaoToken::ProgramEnd).value(), 6);
        assert_eq!(GingerToken::Mao(MaoToken::Fn).value(), 7);
        assert_eq!(GingerToken::Mao(MaoToken::Struct).value(), 8);

        // Test wrapped SingleToken - offset by 5 + MaoToken::COUNT
        assert_eq!(GingerToken::Single(SingleToken::Single).value(), 9);
    }

    #[test]
    fn test_from_conversions() {
        // Test MaoToken -> GingerToken conversion using From::from()
        let mao_start = GingerToken::from(MaoToken::ProgramStart);
        assert_eq!(mao_start.value(), 5);

        let mao_end = GingerToken::from(MaoToken::ProgramEnd);
        assert_eq!(mao_end.value(), 6);

        // Test SingleToken -> GingerToken conversion
        let single = GingerToken::from(SingleToken::Single);
        assert_eq!(single.value(), 9);
        
        // .into() still works automatically thanks to From implementation
        let mao_via_into: GingerToken = MaoToken::ProgramStart.into();
        assert_eq!(mao_via_into.value(), 5);
        
        // Both syntaxes work
        assert_eq!(GingerToken::from(MaoToken::Struct).value(), 8);
        assert_eq!(Into::<GingerToken>::into(MaoToken::Fn).value(), 7);
    }

    #[test]
    fn test_token_counts() {
        assert_eq!(MaoToken::COUNT, 4);
        assert_eq!(SingleToken::COUNT, 1);
        assert_eq!(GingerToken::COUNT, 10); // 4 (Mao) + 5 (native) + 1 (Single)
    }
}
