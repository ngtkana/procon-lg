#!/bin/bash

# Run examples and manage expected outputs
# This script can update expected outputs, show diffs, or perform dry-runs

set -e

# Color definitions (for terminal output)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default settings
DRY_RUN=false
SHOW_DIFF=false
LIST_CHANGED=false
FAIL_ON_DIFF=false
UPDATE_MODE=true

# Statistics
TOTAL_EXAMPLES=0
UNCHANGED_COUNT=0
CHANGED_COUNT=0
FAILED_COUNT=0
CHANGED_EXAMPLES=()
FAILED_EXAMPLES=()

# Function to show usage
show_help() {
    cat << EOF
Usage: $0 [OPTIONS]

Run examples and manage their expected outputs.

OPTIONS:
    -n, --dry-run          Run simulation without updating files
    -d, --diff             Show differences in diff format
    -l, --list-changed     List files that have changes
    -f, --fail-on-diff     Exit with error if differences found
    -h, --help             Show this help message

EXAMPLES:
    $0                     Update all expected outputs (default)
    $0 --dry-run --diff    Show what would change without updating
    $0 --dry-run --list-changed --fail-on-diff
                          List changed files and fail if any found

Default behavior: Update expected output files for all examples.
EOF
}

# Function to parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -n|--dry-run)
                DRY_RUN=true
                UPDATE_MODE=false
                shift
                ;;
            -d|--diff)
                SHOW_DIFF=true
                shift
                ;;
            -l|--list-changed)
                LIST_CHANGED=true
                shift
                ;;
            -f|--fail-on-diff)
                FAIL_ON_DIFF=true
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                echo "[ERROR] Unknown option: $1" >&2
                echo "Use --help for usage information." >&2
                exit 1
                ;;
        esac
    done
}

# Function to log messages with prefixes
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Function to build examples
build_examples() {
    log_info "Building examples..."
    if ! cargo build --examples --quiet; then
        log_error "Failed to build examples"
        exit 1
    fi
}

# Function to process a single example
process_example() {
    local example="$1"
    local executable="target/debug/examples/$example"
    local expected_file="tests/expected_outputs/${example}.out"
    
    log_info "Processing example: $example"
    
    # Check if executable exists
    if [ ! -f "$executable" ]; then
        log_error "Executable not found: $executable"
        FAILED_EXAMPLES+=("$example")
        ((FAILED_COUNT++))
        return 1
    fi
    
    # Debug: Show executable info
    log_info "Executable details: $(ls -la "$executable")"
    log_info "File type: $(file "$executable")"
    
    # Generate actual output
    local actual_output
    actual_output=$(mktemp)
    log_info "Running: $executable"
    if ! "$executable" > "$actual_output" 2>&1; then
        local exit_code=$?
        log_error "Failed to run: $executable (exit code: $exit_code)"
        if [ -s "$actual_output" ]; then
            log_error "Error output: $(cat "$actual_output")"
        fi
        FAILED_EXAMPLES+=("$example")
        ((FAILED_COUNT++))
        rm "$actual_output"
        return 1
    fi
    log_info "Successfully executed: $executable"
    
    # Compare with expected output if it exists
    local has_changes=false
    if [ -f "$expected_file" ]; then
        if ! diff -q "$expected_file" "$actual_output" > /dev/null 2>&1; then
            has_changes=true
        fi
    else
        # New expected output file
        has_changes=true
        log_warn "Expected output file not found, will be created: $expected_file"
    fi
    
    if [ "$has_changes" = true ]; then
        CHANGED_EXAMPLES+=("$example")
        ((CHANGED_COUNT++))
        
        if [ "$SHOW_DIFF" = true ] && [ -f "$expected_file" ]; then
            echo ""
            echo "--- $expected_file"
            echo "+++ $example (actual)"
            diff -u "$expected_file" "$actual_output" || true
            echo ""
        fi
        
        if [ "$LIST_CHANGED" = true ]; then
            log_warn "Output differs: $example"
        fi
        
        if [ "$UPDATE_MODE" = true ]; then
            # Create directory if it doesn't exist
            mkdir -p "$(dirname "$expected_file")"
            cp "$actual_output" "$expected_file"
            log_success "Updated: $expected_file"
        fi
    else
        ((UNCHANGED_COUNT++))
        if [ "$LIST_CHANGED" = false ] && [ "$SHOW_DIFF" = false ]; then
            log_info "No changes: $example"
        fi
    fi
    
    rm "$actual_output"
    return 0
}

# Function to report final results
report_results() {
    echo ""
    log_info "Summary: $UNCHANGED_COUNT unchanged, $CHANGED_COUNT changed, $FAILED_COUNT failed"
    
    if [ ${#CHANGED_EXAMPLES[@]} -gt 0 ]; then
        if [ "$UPDATE_MODE" = true ]; then
            log_success "Updated examples: ${CHANGED_EXAMPLES[*]}"
        else
            log_warn "Examples with changes: ${CHANGED_EXAMPLES[*]}"
        fi
    fi
    
    if [ ${#FAILED_EXAMPLES[@]} -gt 0 ]; then
        log_error "Failed examples: ${FAILED_EXAMPLES[*]}"
    fi
}

# Main function
main() {
    # Move to project root
    cd "$(dirname "$0")/.."
    
    # Parse command line arguments
    parse_arguments "$@"
    
    # Build examples
    build_examples
    
    # Get all example files
    local examples=()
    while IFS= read -r -d '' file; do
        local basename
        basename=$(basename "$file" .rs)
        examples+=("$basename")
    done < <(find examples -name "*.rs" -print0)
    
    if [ ${#examples[@]} -eq 0 ]; then
        log_error "No examples found in examples/ directory"
        exit 1
    fi
    
    TOTAL_EXAMPLES=${#examples[@]}
    log_info "Found ${TOTAL_EXAMPLES} examples: ${examples[*]}"
    
    if [ "$DRY_RUN" = true ]; then
        log_info "Running in dry-run mode (no files will be updated)"
    fi
    
    # Process each example
    for example in "${examples[@]}"; do
        process_example "$example"
    done
    
    # Report results
    report_results
    
    # Determine exit code
    local exit_code=0
    
    if [ ${#FAILED_EXAMPLES[@]} -gt 0 ]; then
        exit_code=1
    elif [ "$FAIL_ON_DIFF" = true ] && [ ${#CHANGED_EXAMPLES[@]} -gt 0 ]; then
        log_error "Differences found and --fail-on-diff specified"
        exit_code=1
    fi
    
    # Run verification if we updated files
    if [ "$UPDATE_MODE" = true ] && [ "$exit_code" = 0 ] && [ ${#CHANGED_EXAMPLES[@]} -gt 0 ]; then
        echo ""
        log_info "Running verification test..."
        if ./scripts/test_examples.sh > /dev/null 2>&1; then
            log_success "All tests passed after update"
        else
            log_error "Some tests failed after update"
            exit_code=1
        fi
    fi
    
    exit $exit_code
}

# Run main function with all arguments
main "$@"
