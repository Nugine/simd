#!/bin/bash -e
COMMIT_HASH=`git rev-parse --short HEAD`
echo "COMMIT: $COMMIT_HASH"
echo

echo "Rust:"
rustc -V -v
echo

echo "System:"
uname -a
echo

echo "CPU:"
grep 'model name' /proc/cpuinfo  | uniq
echo
