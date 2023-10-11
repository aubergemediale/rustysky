# rustysky

![Rust Tests](https://github.com/YOUR_GITHUB_USERNAME/YOUR_REPOSITORY_NAME/workflows/Rust/badge.svg)

rustysky is a Rust client library for the bluesky social network. Along with the library, this project also provides a CLI tool and examples to help users get started with the bluesky API.

## Table of Contents

- [Installation](#installation)
- [Usage](#usage)
  - [Using the Library](#using-the-library)
  - [CLI Tool](#cli-tool)
  - [Examples](#examples)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)

## Installation

To include the `rustysky` library in your Rust project, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
rustysky = "0.1.0"  # Replace with the latest version
```

Then run:

```
cargo build
```

## Usage

### Using the Library

To use the `rustysky` library in your Rust project:

```rust
extern crate rustysky;

// Your code here...
```

### CLI Tool

To run the `rustysky_cli`:

```
cargo run --bin rustysky_cli
```

### Examples

To demonstrate the usage of `rustysky`, we've provided some examples:

1. **Auth Example:**

   Run the auth example with:

   ```
   cargo run --example auth
   ```

2. **No-Auth Example:**

   Run the no-auth example with:

   ```
   cargo run --example no_auth
   ```

## Testing

### Unit Tests

Unit tests are co-located with the code they test. To run only the unit tests for the library:

```
cargo test --lib
```

### Integration Tests

Integration tests are located in the `tests` directory. To run the integration tests:

```
cargo test --test bsky_agent_tests
```

### Running All Tests

To run both unit and integration tests together:

```
cargo test
```
```

## License

This project is licensed under the [MIT License](LICENSE.md).
```
