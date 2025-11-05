# procon-lg

A procedural macro library for debugging recursive functions in competitive programming.

## Development Environment Setup

### Pre-commit Configuration

To run the same quality checks locally as in the CI environment, you can use pre-commit.

```bash
# Install pre-commit
pip3 install pre-commit

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

### Early Return Support

The macro supports early returns in recursive functions:

```rust
#[lg_recur(show_return)]
fn binary_search(arr: &[i32], target: i32, left: usize, right: usize) -> Option<usize> {
    if left > right {
        return None; // early return - logged automatically
    }

    let mid = left + (right - left) / 2;

    if arr[mid] == target {
        return Some(mid); // early return - logged automatically
    } else if arr[mid] < target {
        binary_search(arr, target, mid + 1, right)
    } else {
        binary_search(arr, target, left, mid - 1)
    }
}
```

## Running Tests

```bash
precommit run --all-files
```
