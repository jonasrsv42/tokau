use crate::space::{Position, TokenSpace};
use crate::token::NameToken;

// Extension trait for filtering iterables by token type
pub trait TokenFilter: Iterator<Item = u32> + Sized {
    fn dynamics<S: TokenSpace>(self) -> impl Iterator<Item = u32> {
        self.filter_map(|id| S::dynamic(id).map(|_| id))
    }

    fn is<S: TokenSpace, T: NameToken>(self) -> impl Iterator<Item = T>
    where
        S: Position<T>,
        T: TryFrom<u32>,
    {
        self.filter_map(|id| S::is::<T>(id))
    }
}

// Implementation for all iterators over u32
impl<I: Iterator<Item = u32> + Sized> TokenFilter for I {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space::tests::DynamicGingerSpace;
    use crate::token::tests::*;

    #[test]
    fn test_token_filter_extension() {
        let tokens: Vec<u32> = vec![0, 5, 6, 7, 10, 50, 1010, 1011, 1200, 1600];

        // Filter to only dynamic tokens (no bounds checking now)
        let dynamic_tokens: Vec<u32> = tokens
            .clone()
            .into_iter()
            .dynamics::<DynamicGingerSpace>()
            .collect();
        assert_eq!(dynamic_tokens, vec![1010, 1011, 1200, 1600]); // All tokens >= RESERVED

        // Filter to only MaoTokens (returns actual token instances)
        let mao_tokens: Vec<MaoToken> = tokens
            .clone()
            .into_iter()
            .is::<DynamicGingerSpace, MaoToken>()
            .collect();
        assert_eq!(
            mao_tokens,
            vec![
                MaoToken::ProgramStart, // from token 5
                MaoToken::ProgramEnd,   // from token 6
                MaoToken::Fn,           // from token 7
            ]
        );
    }

    #[test]
    fn test_stacking_operations() {
        let tokens: Vec<u32> = vec![0, 1, 5, 6, 7, 8, 9, 10, 50, 1010, 1100, 1200, 1500, 2000];

        // Stack operations: first filter to dynamic, then take only first 3
        let stacked: Vec<u32> = tokens
            .clone()
            .into_iter()
            .dynamics::<DynamicGingerSpace>()
            .take(3)
            .collect();
        assert_eq!(stacked, vec![1010, 1100, 1200]);

        // Chain different filters
        let all_special_tokens: Vec<u32> = tokens
            .clone()
            .into_iter()
            .is::<DynamicGingerSpace, GingerToken>()
            .map(|token| token.inside::<DynamicGingerSpace>())
            .chain(
                tokens
                    .clone()
                    .into_iter()
                    .is::<DynamicGingerSpace, MaoToken>()
                    .map(|token| token.inside::<DynamicGingerSpace>()),
            )
            .collect();
        assert_eq!(all_special_tokens, vec![0, 1, 5, 6, 7, 8]);

        // Filter and then count
        let mao_count = tokens
            .clone()
            .into_iter()
            .is::<DynamicGingerSpace, MaoToken>()
            .count();
        assert_eq!(mao_count, 4); // ProgramStart, ProgramEnd, Fn, Struct
    }

    #[test]
    fn test_edge_cases() {
        // Empty iterator
        let empty: Vec<u32> = vec![];
        let empty_result: Vec<u32> = empty.into_iter().dynamics::<DynamicGingerSpace>().collect();
        assert_eq!(empty_result, vec![]);

        // All tokens in dynamic range (no upper bounds)
        let out_of_range = vec![2000, 3000, 4000];
        let no_dynamics: Vec<u32> = out_of_range
            .clone()
            .into_iter()
            .dynamics::<DynamicGingerSpace>()
            .collect();
        assert_eq!(no_dynamics, vec![2000, 3000, 4000]); // All tokens >= RESERVED

        let no_names: Vec<MaoToken> = out_of_range
            .into_iter()
            .is::<DynamicGingerSpace, MaoToken>()
            .collect();
        assert_eq!(no_names, vec![]);

        // Boundary cases
        let boundary = vec![1009, 1010, 1509, 1510]; // Last static, first tail, last tail, out of range
        let dynamic_boundary: Vec<u32> = boundary
            .clone()
            .into_iter()
            .dynamics::<DynamicGingerSpace>()
            .collect();
        assert_eq!(dynamic_boundary, vec![1010, 1509, 1510]); // All dynamic tokens (no bounds checking)
    }
}
