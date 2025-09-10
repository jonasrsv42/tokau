# Tokau

A Rust library for compile-time type-safe token space management, designed for working with Large Language Models (LLMs) and other tokenized systems.

## Overview

Tokau provides **compile-time support for token ranges** with type safety, preventing common offset-related bugs when working with multiple token types in combined spaces. The library ensures that tokens are always positioned correctly within their designated spaces through Rust's type system.

## Problem

When working with Large Language Models, you often need to combine different token types into a single token space:
- Control tokens (0-9)  
- Language-specific tokens (10-99)
- Text vocabulary (100-999)
- Dynamic vocabulary (1000+)

Without proper tooling, it's easy to accidentally use raw token values without applying the correct offsets, leading to tokens mapping to wrong positions in your combined space.

## Solution

Tokau prevents these bugs through:

1. **Compile-time type safety**: Tokens can only be positioned within spaces that explicitly support them
2. **Automatic offset management**: The type system ensures offsets are always applied correctly
3. **Type-safe iterators**: Filter and process token sequences with compile-time guarantees
4. **Mixed static/dynamic support**: Handle both fixed token counts and unbounded vocabularies

## Core Concepts

### Tokens

Define your token types using derive macros:

```rust
use tokau::{Name, Token, range};

// Discrete tokens (enums)
#[derive(Name, Debug, Clone, Copy)]
#[repr(u32)]
enum ControlToken {
    Start,
    Stop,
    Pause,
}

// Range tokens (continuous ranges)
#[derive(Debug, PartialEq)]
#[range(1000)]
struct TextTokens(u32);
```

### Token Spaces

Define how token types are arranged:

```rust
use tokau::{Space, TokenSpace, Position};

#[derive(Space, Debug, PartialEq)]
enum MyTokenSpace {
    Control(ControlToken),     // Positions 0-2
    Text(TextTokens),          // Positions 3-1002
    #[dynamic]
    Vocab(u32),                // Positions 1003+
}
```

### Type-Safe Operations

```rust
// Get token positions safely
let position = MyTokenSpace::position_of(ControlToken::Start); // 0
let position = MyTokenSpace::position_of(TextTokens(42));      // 45

// Convert positions back to tokens
let token = MyTokenSpace::try_as::<ControlToken>(0); // Some(ControlToken::Start)
let token = MyTokenSpace::try_as::<TextTokens>(100);  // Some(TextTokens(97))

// Handle dynamic tokens
let remainder = MyTokenSpace::remainder(1500); // Some(497)

// Decode token IDs to space tokens
let space_token = MyTokenSpace::try_from(0); // Ok(MyTokenSpace::Control(ControlToken::Start))
```

### Iterator Extensions

```rust
use tokau::TokenFilter;

let token_ids = vec![0, 1, 50, 100, 1010, 2000];

// Get remainder values for dynamic tokens
let remainders: Vec<u32> = token_ids.clone()
    .into_iter()
    .remainders::<MyTokenSpace>()
    .collect(); // [7, 997]

// Decode all tokens
let decoded: Vec<Result<MyTokenSpace, TokauError>> = token_ids
    .into_iter()
    .decode::<MyTokenSpace>()
    .collect();
```

## Current Limitations

**⚠️ Code-Data Coupling**: Tokau currently has tight coupling between code and data. Token space definitions are compile-time constructs, which means:

- **Static versioning only**: You can handle multiple model versions, but only if they're explicitly versioned in code (e.g., `MaoTokenV1`, `MaoTokenV2`, `SpaceAlpha`, `SpaceBeta`)
- **No dynamic versioning**: Cannot have multiple versions of the same named token space (e.g., multiple `MaoToken` definitions)
- **Model compatibility**: Each exported model must match exactly one compile-time token space definition
- **Runtime flexibility**: Cannot dynamically load models with unknown token space layouts

**What works**: Multiple model versions with different token spaces, as long as each has a distinct compile-time definition.

**What doesn't work**: Loading models at runtime where the token space structure is unknown at compile time, or having multiple versions of identically-named token types.

Despite this limitation, Tokau provides significant value for development-time safety and correctness when working with known token space layouts.

## Benefits

- **Compile-time safety**: Mismatched token types and spaces are caught at compile time  
- **Zero runtime overhead**: All safety checks happen during compilation
- **Ergonomic API**: Extension traits make working with token sequences natural
- **Flexible design**: Supports static tokens, range tokens, and dynamic vocabularies
- **Error handling**: Rich error types with context for debugging

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tokau = { git = "ssh://git@github.com/jonasrsv42/tokau", features = ["derive"] }

```

The `derive` feature (enabled by default) provides the `#[derive(Name)]`, `#[range(N)]`, and `#[derive(Space)]` macros.

## Examples

### Basic Token Space

```rust
use tokau::{Name, Space, TokenSpace, TokauError, range};

#[derive(Name, Debug, Clone, Copy)]
#[repr(u32)]
enum CommandToken {
    Execute,
    Cancel,
}

#[derive(Debug, PartialEq)]
#[range(100)]
struct DataTokens(u32);

#[derive(Space, Debug, PartialEq)]
enum SimpleSpace {
    Command(CommandToken),
    Data(DataTokens),
}

fn main() -> Result<(), TokauError> {
    // Type-safe positioning
    assert_eq!(SimpleSpace::position_of(CommandToken::Execute), 0);
    assert_eq!(SimpleSpace::position_of(DataTokens(42)), 44); // 2 + 42

    // Safe decoding
    let decoded = SimpleSpace::try_from(1)?;
    assert_eq!(decoded, SimpleSpace::Command(CommandToken::Cancel));

    Ok(())
}
```

### Multiple Spaces with Shared Tokens

```rust
use tokau::{Name, Space, TokenSpace};

#[derive(Name, Debug, Clone, Copy)]
#[repr(u32)]
enum SharedToken {
    Alpha,
    Beta,
}

// Same token type in different positions
#[derive(Space, Debug, PartialEq)]
enum SpaceA {
    Shared(SharedToken),  // Positions 0-1
}

#[derive(Space, Debug, PartialEq)]
enum SpaceB {
    Other(u32),           // Position 0
    Shared(SharedToken),  // Positions 1-2
}

fn main() {
    // Same token, different positions in different spaces
    assert_eq!(SpaceA::position_of(SharedToken::Alpha), 0);
    assert_eq!(SpaceB::position_of(SharedToken::Alpha), 1);
}
```

## Documentation

See the `tests/` directory for comprehensive examples covering:
- Complex token space layouts
- Dynamic vocabulary handling
- Iterator operations
- Error handling patterns
- Token reuse across multiple spaces
