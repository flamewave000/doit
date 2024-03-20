#!/bin/bash

echo "[cargo check]"
if ! cargo check ; then
	exit 1
fi
echo "[cargo clippy --all-targets --all-features -- -D warnings]"
if ! cargo clippy --all-targets --all-features -- -D warnings ; then
	exit 1
fi
echo -n "[cargo test -q]"
if ! cargo test -q ; then
	exit 1
fi
exit 0
