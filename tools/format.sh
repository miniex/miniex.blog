#!/usr/bin/env bash

set -euo pipefail

errors=""
failed=0
total=0
done_count=0

# Count HTML template files
html_files=()
while IFS= read -r f; do html_files+=("$f"); done < <(find templates -name '*.html' | sort)
total=$(( 2 + ${#html_files[@]} ))

print_progress() {
    local pct=$(( done_count * 100 / total ))
    local bar_len=30
    local filled=$(( pct * bar_len / 100 ))
    local empty=$(( bar_len - filled ))
    printf "\r  \033[36m[%-${bar_len}s]\033[0m %d/%d %s\033[K" \
        "$(printf '%*s' "$filled" '' | tr ' ' '█')$(printf '%*s' "$empty" '' | tr ' ' '░')" \
        "$done_count" "$total" "$1"
}

# ── Rust fmt ──
print_progress "cargo fmt"
fmt_out=$(cargo fmt --all 2>&1) || {
    errors+="── cargo fmt ──"$'\n'"$fmt_out"$'\n\n'
    failed=$((failed + 1))
}
done_count=$((done_count + 1))

# ── CSS/JS prettier ──
print_progress "prettier (css/js)"
prettier_out=$(npx prettier --write --ignore-path /dev/null "assets/styles/**/*.css" "!assets/styles/tailwind.output.css" "assets/js/**/*.js" 2>&1) || {
    errors+="── prettier (css/js) ──"$'\n'"$prettier_out"$'\n\n'
    failed=$((failed + 1))
}
done_count=$((done_count + 1))

# ── HTML templates (prettier, skip files with Askama string comparisons) ──
for file in "${html_files[@]}"; do
    name="${file#templates/}"
    print_progress "$name"
    if grep -qE '\{%.*==.*"' "$file"; then
        done_count=$((done_count + 1))
        continue
    fi
    html_out=$(npx prettier --write --parser html "$file" 2>&1) || {
        errors+="── $name ──"$'\n'"$html_out"$'\n\n'
        failed=$((failed + 1))
    }
    done_count=$((done_count + 1))
done

# ── Result ──
if [ "$failed" -gt 0 ]; then
    printf "\r  \033[31m✗ %d error(s)\033[0m%*s\n\n" "$failed" 40 ""
    printf "%s" "$errors"
    exit 1
else
    printf "\r  \033[32m✓ formatted %d targets\033[0m%*s\n" "$total" 40 ""
fi
