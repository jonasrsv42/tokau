# Tokau

A Rust library for handling LLM token spaces with type safety to reduce the chance of errors when working with token ranges.

## Problem

When working with Large Language Models (LLMs), you often have different token ranges defined as enums in different files. When combining these into a single token space with offsets, it becomes error-prone to accidentally refer to individual token ranges without accounting for their proper offsets in the combined space.

For example, if you have:
- Special tokens (0-9)  
- Language tokens (10-99)
- Text vocabulary (100+)

It's easy to accidentally use a language token value directly instead of adding the proper offset, leading to bugs where tokens map to the wrong values in your combined token space.

## Solution

Tokau provides a type-safe trait system that prevents these offset-related bugs by:

1. **Type-safe token positioning**: Tokens can only be positioned within spaces that explicitly support them
2. **Compile-time offset validation**: The type system ensures offsets are always applied correctly
3. **Space-aware filtering**: Easily filter token arrays by type within specific token spaces
4. **Mixed static/dynamic support**: Handle both fixed token counts and dynamic vocabularies

## Core Concepts

### Token Types

- **`NameToken`**: Discrete tokens with specific enum values (e.g., `StartToken`, `EndToken`)
- **`RangeToken`**: Contiguous ranges without specific instances (e.g., text vocabulary)

### Token Spaces

Token spaces define how different token types are laid out with proper offsets:

```rust
// Define your token space
impl TokenSpace for MySpace {
    const RESERVED: u32 = 100; // Static tokens
    
    fn count(&self) -> u32 {
        Self::RESERVED + self.vocab_size // Total including dynamic
    }
}

// Position token types within the space
impl Position<SpecialToken> for MySpace {
    const OFFSET: u32 = 0; // Special tokens start at 0
}

impl Position<TextTokens> for MySpace {
    const OFFSET: u32 = 50; // Text tokens start at 50
}
```

### Type-Safe Usage

```rust
// Get positioned token values safely
let token = SpecialToken::Start;
let positioned_value = token.in_::<MySpace>(); // Automatically applies offset

// Filter token arrays by type
let tokens: Vec<u32> = vec![0, 1, 50, 51, 200];
let special_tokens: Vec<SpecialToken> = tokens
    .into_iter()
    .specials::<MySpace, SpecialToken>()
    .collect();
```

## Benefits

- **Prevents offset bugs**: The type system ensures you can't accidentally use raw token values
- **Compile-time safety**: Mismatched token types and spaces are caught at compile time  
- **Ergonomic filtering**: Extension methods make it easy to work with mixed token arrays
- **Flexible design**: Supports both static and dynamic token spaces
- **Zero runtime cost**: All safety checks happen at compile time

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
tokau = "0.1.0"

# Or from GitHub with derive macro support:
tokau = { git = "https://github.com/jonasrsv42/tokau", features = ["derive"] }
```

### Using the Derive Macro

The `Name` derive macro automatically implements `Token`, `NameToken`, and `TryFrom<u32>` for your enums:

```rust
use tokau::{Name, NameToken, Token};

#[derive(Name, Debug, Clone, Copy)]
#[repr(u32)]  // Required for the Name derive macro
enum MyToken {
    Start,
    End,
    Process,
}

// Automatically generates:
// - MyToken::COUNT = 3
// - MyToken::Start.value() = 0
// - MyToken::End.value() = 1
// - MyToken::Process.value() = 2
// - TryFrom<u32> implementation
```

The derive feature is enabled by default. If you want to use tokau without the derive macro:

```toml
[dependencies]
tokau = { git = "https://github.com/jonasrsv42/tokau", default-features = false }
```

See the tests in `tests/` for comprehensive examples of defining token types, spaces, and using the filtering APIs.
