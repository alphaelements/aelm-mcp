#!/usr/bin/env bash
# Verify that every Rust source file containing tests includes at least one
# abnormal / error-path test case.  Detected via naming conventions and
# assertion patterns.
#
# Exit 0  — all files OK
# Exit 1  — one or more files lack abnormal tests

set -euo pipefail

ABNORMAL_NAME_RE='fn test_\w*(error|invalid|fail|empty|missing|unknown|reject|bad|malformed|overflow|boundary|panic|negative|zero_|_none|denied)'
ABNORMAL_ASSERT_RE='is_err\(\)|unwrap_err\(\)|, Err\(|should_panic'

exit_code=0
missing_files=()

while IFS= read -r -d '' file; do
    # Only consider files that contain at least one #[test] attribute.
    if ! grep -q '#\[test\]' "$file"; then
        continue
    fi

    has_name=0
    has_assert=0

    grep -cP "$ABNORMAL_NAME_RE" "$file" > /dev/null 2>&1 && has_name=1
    grep -cP "$ABNORMAL_ASSERT_RE" "$file" > /dev/null 2>&1 && has_assert=1

    if [ "$has_name" -eq 0 ] && [ "$has_assert" -eq 0 ]; then
        missing_files+=("$file")
        exit_code=1
    fi
done < <(find src/ tests/ -name '*.rs' -not -path '*/target/*' -print0 2>/dev/null)

if [ "$exit_code" -ne 0 ]; then
    echo "MISSING ABNORMAL/ERROR TESTS in the following files:"
    for f in "${missing_files[@]}"; do
        echo "  $f"
    done
    echo ""
    echo "Each file with #[test] must include at least one error/edge-case test."
    echo "Use naming like test_*_error_*, test_*_invalid_*, test_*_empty_*, etc."
    echo "Or include assertions such as is_err(), unwrap_err(), Err(...)."
fi

exit $exit_code
