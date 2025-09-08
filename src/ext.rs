use crate::error::TokauError;
use crate::space::{Position, TokenSpace};
use crate::token::Token;

// Extension trait for filtering iterables by token type
pub trait TokenFilter: Iterator<Item = u32> + Sized {
    fn remainders<S: TokenSpace>(self) -> impl Iterator<Item = u32> {
        self.filter_map(|id| S::remainder(id))
    }

    fn try_as<S: TokenSpace, T: Token>(self) -> impl Iterator<Item = T>
    where
        S: Position<T>,
        T: TryFrom<u32, Error = TokauError>,
    {
        self.filter_map(|id| S::try_as::<T>(id))
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

        // Filter to remainder values of dynamic tokens
        let remainder_values: Vec<u32> = tokens
            .clone()
            .into_iter()
            .remainders::<DynamicGingerSpace>()
            .collect();
        assert_eq!(remainder_values, vec![0, 1, 190, 590]); // Remainder values (token_id - RESERVED)

        // Filter to only MaoTokens (returns actual token instances)
        let mao_tokens: Vec<MaoToken> = tokens
            .clone()
            .into_iter()
            .try_as::<DynamicGingerSpace, MaoToken>()
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

        // Stack operations: first filter to remainder values, then take only first 3
        let stacked: Vec<u32> = tokens
            .clone()
            .into_iter()
            .remainders::<DynamicGingerSpace>()
            .take(3)
            .collect();
        assert_eq!(stacked, vec![0, 90, 190]);

        // Chain different filters
        let all_special_tokens: Vec<u32> = tokens
            .clone()
            .into_iter()
            .try_as::<DynamicGingerSpace, GingerToken>()
            .map(|token| DynamicGingerSpace::position_of(token))
            .chain(
                tokens
                    .clone()
                    .into_iter()
                    .try_as::<DynamicGingerSpace, MaoToken>()
                    .map(|token| DynamicGingerSpace::position_of(token)),
            )
            .collect();
        assert_eq!(all_special_tokens, vec![0, 1, 5, 6, 7, 8]);

        // Filter and then count
        let mao_count = tokens
            .clone()
            .into_iter()
            .try_as::<DynamicGingerSpace, MaoToken>()
            .count();
        assert_eq!(mao_count, 4); // ProgramStart, ProgramEnd, Fn, Struct
    }

    #[test]
    fn test_edge_cases() {
        // Empty iterator
        let empty: Vec<u32> = vec![];
        let empty_result: Vec<u32> = empty
            .into_iter()
            .remainders::<DynamicGingerSpace>()
            .collect();
        assert_eq!(empty_result, vec![]);

        // All tokens in dynamic range (no upper bounds)
        let out_of_range = vec![2000, 3000, 4000];
        let remainder_values: Vec<u32> = out_of_range
            .clone()
            .into_iter()
            .remainders::<DynamicGingerSpace>()
            .collect();
        assert_eq!(remainder_values, vec![990, 1990, 2990]); // Remainder values (token_id - RESERVED)

        let no_names: Vec<MaoToken> = out_of_range
            .into_iter()
            .try_as::<DynamicGingerSpace, MaoToken>()
            .collect();
        assert_eq!(no_names, vec![]);

        // Boundary cases
        let boundary = vec![1009, 1010, 1509, 1510]; // Last static, first dynamic, dynamic tokens
        let remainder_boundary: Vec<u32> = boundary
            .clone()
            .into_iter()
            .remainders::<DynamicGingerSpace>()
            .collect();
        assert_eq!(remainder_boundary, vec![0, 499, 500]); // Remainder values (excluding 1009 which is static)
    }
}
