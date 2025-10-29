#!/bin/bash

# examples の出力をテストするスクリプト

set -e

# 色の定義
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# プロジェクトルートに移動
cd "$(dirname "$0")/.."

echo "🔧 Building examples..."
cargo build --examples

# テスト結果
FAILED_TESTS=()
PASSED_TESTS=()

# 各exampleをテスト
for example in sum fib gcd countdown no_debug custom_fmt; do
    echo "🧪 Testing example: $example"
    
    # 期待される出力ファイルが存在するかチェック
    expected_file="tests/expected_outputs/${example}.out"
    if [ ! -f "$expected_file" ]; then
        echo -e "${RED}❌ Expected output file not found: $expected_file${NC}"
        FAILED_TESTS+=("$example")
        continue
    fi
    
    # 実行ファイルが存在するかチェック
    executable="target/debug/examples/$example"
    if [ ! -f "$executable" ]; then
        echo -e "${RED}❌ Executable not found: $executable${NC}"
        FAILED_TESTS+=("$example")
        continue
    fi
    
    # 実際の出力を取得
    actual_output=$(mktemp)
    "$executable" > "$actual_output" 2>&1
    
    # 期待される出力と比較
    if diff -u "$expected_file" "$actual_output" > /dev/null; then
        echo -e "${GREEN}✅ $example: PASSED${NC}"
        PASSED_TESTS+=("$example")
    else
        echo -e "${RED}❌ $example: FAILED${NC}"
        echo "Differences found:"
        diff -u "$expected_file" "$actual_output" || true
        FAILED_TESTS+=("$example")
    fi
    
    # 一時ファイルを削除
    rm "$actual_output"
done

# 結果の報告
echo ""
echo "📊 Test Results:"
echo -e "${GREEN}✅ Passed: ${#PASSED_TESTS[@]} tests${NC}"
if [ ${#PASSED_TESTS[@]} -gt 0 ]; then
    for test in "${PASSED_TESTS[@]}"; do
        echo -e "  ${GREEN}✅ $test${NC}"
    done
fi

echo -e "${RED}❌ Failed: ${#FAILED_TESTS[@]} tests${NC}"
if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    for test in "${FAILED_TESTS[@]}"; do
        echo -e "  ${RED}❌ $test${NC}"
    done
fi

# 失敗したテストがある場合は非ゼロの終了コードを返す
if [ ${#FAILED_TESTS[@]} -gt 0 ]; then
    echo ""
    echo -e "${RED}Some tests failed. Please check the differences above.${NC}"
    exit 1
else
    echo ""
    echo -e "${GREEN}🎉 All tests passed!${NC}"
    exit 0
fi
