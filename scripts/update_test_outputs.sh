#!/bin/bash

# Test outputs update script
# This script automatically updates expected outputs for all examples

set -e

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Move to project root
cd "$(dirname "$0")/.."

echo -e "${BLUE}🔧 Building examples...${NC}"
cargo build --examples

# Get all example files
EXAMPLES=($(find examples -name "*.rs" -exec basename {} .rs \;))

echo -e "${BLUE}📋 Found examples: ${EXAMPLES[*]}${NC}"

# Create tests/expected_outputs directory if it doesn't exist
mkdir -p tests/expected_outputs

UPDATED_COUNT=0
FAILED_COUNT=0
FAILED_EXAMPLES=()

# Update expected outputs for each example
for example in "${EXAMPLES[@]}"; do
    echo -e "${YELLOW}🔄 Updating expected output for: $example${NC}"
    
    # Check if executable exists
    executable="target/debug/examples/$example"
    if [ ! -f "$executable" ]; then
        echo -e "${RED}❌ Executable not found: $executable${NC}"
        FAILED_EXAMPLES+=("$example")
        ((FAILED_COUNT++))
        continue
    fi
    
    # Generate new expected output
    output_file="tests/expected_outputs/${example}.out"
    if "$executable" > "$output_file" 2>&1; then
        echo -e "${GREEN}✅ Updated: $output_file${NC}"
        ((UPDATED_COUNT++))
    else
        echo -e "${RED}❌ Failed to run: $executable${NC}"
        FAILED_EXAMPLES+=("$example")
        ((FAILED_COUNT++))
    fi
done

echo ""
echo -e "${BLUE}📊 Update Results:${NC}"
echo -e "${GREEN}✅ Successfully updated: $UPDATED_COUNT examples${NC}"
echo -e "${RED}❌ Failed to update: $FAILED_COUNT examples${NC}"

if [ ${#FAILED_EXAMPLES[@]} -gt 0 ]; then
    echo -e "${RED}Failed examples:${NC}"
    for example in "${FAILED_EXAMPLES[@]}"; do
        echo -e "  ${RED}❌ $example${NC}"
    done
fi

echo ""
echo -e "${BLUE}🧪 Running tests to verify updates...${NC}"

# Run test script to verify the updates
if ./scripts/test_examples.sh; then
    echo ""
    echo -e "${GREEN}🎉 All tests passed! Expected outputs have been successfully updated.${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}⚠️  Some tests failed after update. Please check the test results above.${NC}"
    exit 1
fi
