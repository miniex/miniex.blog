#!/usr/bin/env bash

set -euo pipefail

errors=""
failed=0
total=0
done_count=0

# Count JS files
js_files=()
while IFS= read -r f; do js_files+=("$f"); done < <(find assets/js -name '*.js' | sort)
total=$(( 2 + ${#js_files[@]} ))

print_progress() {
    local pct=$(( done_count * 100 / total ))
    local bar_len=30
    local filled=$(( pct * bar_len / 100 ))
    local empty=$(( bar_len - filled ))
    printf "\r  \033[36m[%-${bar_len}s]\033[0m %d/%d %s\033[K" \
        "$(printf '%*s' "$filled" '' | tr ' ' '█')$(printf '%*s' "$empty" '' | tr ' ' '░')" \
        "$done_count" "$total" "$1"
}

# ── Rust clippy ──
print_progress "clippy"
clippy_out=$(cargo clippy --workspace --all-targets -- -D warnings 2>&1) || {
    errors+="── clippy ──"$'\n'"$clippy_out"$'\n\n'
    failed=$((failed + 1))
}
done_count=$((done_count + 1))

# ── Tailwind CSS build check ──
print_progress "tailwind build"
tw_out=$(npx tailwindcss -i ./assets/styles/tailwind.input.css -o ./assets/styles/tailwind.output.css 2>&1) || {
    errors+="── tailwind build ──"$'\n'"$tw_out"$'\n\n'
    failed=$((failed + 1))
}
done_count=$((done_count + 1))

# ── JS syntax check (node --check) ──
for file in "${js_files[@]}"; do
    name="${file#assets/js/}"
    print_progress "$name"
    js_out=$(node --check "$file" 2>&1) || {
        errors+="── $name ──"$'\n'"$js_out"$'\n\n'
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
    printf "\r  \033[32m✓ all %d checks passed\033[0m%*s\n" "$total" 40 ""
fi
