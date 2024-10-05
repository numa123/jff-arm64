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

assert 3 "1+2"
assert 5 "1+2+2"
assert 2 "4-2"
assert 1 "4-2-1"
assert 21 "5+20-4"

echo OK