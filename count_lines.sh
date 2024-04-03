#!/bin/sh

echo "Rust:"
find . -type f -name "*.rs" ! -wholename "**/target/*" | xargs wc -l | sort -nr
