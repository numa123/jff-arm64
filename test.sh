#!/bin/bash
assert() {
	expected="$1"
	input="$2"
	./target/debug/jff "$input" > tmp.s
	gcc-14 -o tmp tmp.s
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
assert 6 "1+2+3"
assert 2 "4-2"
assert 1 "4-2-1"
assert 21 "5+20-4"
assert 10 " 2 * 5 "
assert 3 " 12 / 4 "
assert 18 "2+4*4"
assert 15 '5*(9-6)'
assert 4 '(3+5)/2'
assert 10 '-10+20'
assert 10 '- -10'
assert 10 '- - +10'

echo OK