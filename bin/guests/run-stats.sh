#!/usr/bin/env bash

output=$(ziskemu "$@" -X 2>&1)
rc=$?

if [ $rc -ne 0 ]; then
    printf '%s\n' "$output" >&2
    exit $rc
fi

steps=$(printf '%s\n' "$output" | grep -E '^STEPS[[:space:]]' | grep -oE '[0-9,]+' | tr -d ',')
total=$(printf '%s\n' "$output" | grep -E '^TOTAL[[:space:]]' | grep -oE '[0-9,]+' | head -1 | tr -d ',')

printf '{"steps": %s, "total": %s}\n' "$steps" "$total"
