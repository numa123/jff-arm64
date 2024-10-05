#!/bin/bash
assert() {
	expected="$1"
	input="$2"
	./target/debug/jff "$input" > tmp.s
	gcc -o tmp tmp.s
	./tmp
	actual="$?"
	if [ "$actual" = "$expected" ]; then
		echo "$input => $actual"
	else
		echo "$input => $expected expected, but got $actual"
		exit 1
	fi
}
cargo build

assert 0 0
assert 42 42
echo OK