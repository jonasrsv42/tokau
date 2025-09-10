use crate::error::TokauError;
use crate::space::TokenSpace;

// Extension trait for filtering iterables by token type
pub trait TokenIter: Iterator<Item = u32> + Sized {
    /// Filter to remainder values of dynamic tokens (tokens >= RESERVED)
    fn remainders<S: TokenSpace>(self) -> impl Iterator<Item = u32> {
        self.filter_map(|id| S::remainder(id))
    }

    /// Decode token IDs to space tokens, returning Result for each conversion
    fn decode<S: TokenSpace>(self) -> impl Iterator<Item = Result<S, TokauError>> {
        self.map(|id| S::try_from(id))
    }

    /// Shift each value to after the token space's reserved range
    /// This adds RESERVED to each value, placing them in the dynamic token range
    fn after_reserved<S: TokenSpace>(self) -> impl Iterator<Item = u32> {
        self.map(|id| S::after_reserved(id))
    }
}

// Implementation for all iterators over u32
impl<I: Iterator<Item = u32> + Sized> TokenIter for I {}

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

        // Decode all tokens to DynamicGingerSpace
        let decoded: Vec<Result<DynamicGingerSpace, TokauError>> = tokens
            .clone()
            .into_iter()
            .decode::<DynamicGingerSpace>()
            .collect();

        // Check successful decodings
        let successful_decodings: Vec<DynamicGingerSpace> =
            decoded.into_iter().filter_map(Result::ok).collect();

        assert_eq!(successful_decodings.len(), 10); // All tokens should decode successfully

        // Check that we can extract MaoTokens from the decoded results
        let mao_tokens: Vec<MaoToken> = successful_decodings
            .into_iter()
            .filter_map(|space_token| match space_token {
                DynamicGingerSpace::Mao(mao) => Some(mao),
                _ => None,
            })
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

        // Test decode with filtering for specific token types
        let decoded_tokens: Vec<DynamicGingerSpace> = tokens
            .clone()
            .into_iter()
            .decode::<DynamicGingerSpace>()
            .filter_map(Result::ok)
            .collect();

        // Extract positions of GingerTokens and MaoTokens
        let special_token_positions: Vec<u32> = decoded_tokens
            .into_iter()
            .filter_map(|space_token| match space_token {
                DynamicGingerSpace::Ginger(ginger) => Some(DynamicGingerSpace::position_of(ginger)),
                DynamicGingerSpace::Mao(mao) => Some(DynamicGingerSpace::position_of(mao)),
                _ => None,
            })
            .collect();
        assert_eq!(special_token_positions, vec![0, 1, 5, 6, 7, 8]);

        // Filter, decode, and count MaoTokens
        let mao_count = tokens
            .clone()
            .into_iter()
            .decode::<DynamicGingerSpace>()
            .filter_map(Result::ok)
            .filter(|space_token| matches!(space_token, DynamicGingerSpace::Mao(_)))
            .count();
        assert_eq!(mao_count, 4); // ProgramStart, ProgramEnd, Fn, Struct
    }

    #[test]
    fn test_after_reserved() {
        // Test shifting values to after the reserved range
        let values: Vec<u32> = vec![0, 1, 10, 100, 500];

        // Shift to after DynamicGingerSpace's reserved range (RESERVED = 1010)
        let shifted: Vec<u32> = values
            .clone()
            .into_iter()
            .after_reserved::<DynamicGingerSpace>()
            .collect();

        // Each value should be increased by RESERVED (1010)
        assert_eq!(shifted, vec![1010, 1011, 1020, 1110, 1510]);

        // These shifted values should all be in the dynamic range
        for &id in &shifted {
            assert!(!DynamicGingerSpace::is_reserved(id));
            assert!(DynamicGingerSpace::remainder(id).is_some());
        }

        // They should decode as Dynamic tokens
        let decoded: Vec<DynamicGingerSpace> = shifted
            .into_iter()
            .decode::<DynamicGingerSpace>()
            .filter_map(Result::ok)
            .collect();

        for (i, token) in decoded.iter().enumerate() {
            match token {
                DynamicGingerSpace::Dynamic(offset) => {
                    assert_eq!(*offset, values[i]); // The offset should match the original value
                }
                _ => panic!("Expected Dynamic token"),
            }
        }

        // Test chaining: shift then get remainders should give back original values
        let round_trip: Vec<u32> = values
            .clone()
            .into_iter()
            .after_reserved::<DynamicGingerSpace>()
            .remainders::<DynamicGingerSpace>()
            .collect();

        assert_eq!(round_trip, values);
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

        // These should all decode successfully as Dynamic tokens
        let decoded_out_of_range: Vec<Result<DynamicGingerSpace, TokauError>> = out_of_range
            .into_iter()
            .decode::<DynamicGingerSpace>()
            .collect();

        // All should be successful Dynamic tokens
        let successful_dynamic: Vec<DynamicGingerSpace> = decoded_out_of_range
            .into_iter()
            .filter_map(Result::ok)
            .collect();
        assert_eq!(successful_dynamic.len(), 3);

        // All should be Dynamic variants
        for token in successful_dynamic {
            assert!(matches!(token, DynamicGingerSpace::Dynamic(_)));
        }

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
