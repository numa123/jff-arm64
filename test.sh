#!/bin/bash
assert() {
	expected="$1"
	input="$2"
	./target/debug/jff "$input" > tmp.s || exit
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

cargo build -q

assert 16 'main.c'
assert 0 'int main() { int i = 1; --i; return i; }'


echo OK