#!/usr/bin/env bash

set +euo pipefail

file=$1

tempf=$(mktemp)
cat "$file" > "$tempf"
tr -d '\r' < "$tempf" > "$file";