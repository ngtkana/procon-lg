# procon-lg

A procedural macro library for debugging recursive functions in competitive programming.

## Development Environment Setup

### Pre-commit Configuration

To run the same quality checks locally as in the CI environment, you can use pre-commit.

```bash
# Install pre-commit
pip install pre-commit

# Install pre-commit hooks
pre-commit install

# Run pre-commit on all files (initial run)
pre-commit run --all-files
```

## Usage

```rust
use procon_lg::lg_recur;

#[lg_recur]
fn fibonacci(n: u32) -> u32 {
    if n <= 1 {
        1
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
```

For detailed usage instructions, please refer to the sample code in the `examples/` directory.

## Running Tests

```bash
# Unit tests
cargo test

# Example tests
./scripts/test_examples.sh

# All QA checks
pre-commit run --all-files
