#!/bin/bash

assert() {
	expected="$1"
	input="$2"

	./target/debug/jff "$input" > tmp.s
	cc -o tmp tmp.s
	./tmp
	actual="$?"

	if [ "$actual" = "$expected" ]; then
		echo "$input => $actual"
	else
		echo "$input => $expected expected, but got $actual"
		exit 1
	fi
}

cargo build # 最初にビルド

assert 0 "0;"
assert 0 "return 0;"

assert 42 "42;"

assert 1 "a=1;a;"
assert 10 "z=10;z;"

assert 1 "a=1; a;"
assert 2 "a=1; b=2; b;"
assert 21 "a=1; b=2; c=3; d=4; z=a+(b+c)*d; z;"


assert 2 "1+1;"
assert 1 "2-1;"
assert 21 "5+20-4;"
assert 41 " 12 + 34 - 5 ;"

assert 10 " 2 * 5 ;"
assert 3 " 12 / 4 ;"
assert 18 "2+4*4;"

assert 1 "(1);"
assert 14  "2*(3+4);"
assert 17  "(40+4*2) - ((12/4) * 2 * 5 + 1);"

assert 2 "1+++++1;"
assert 0 "1++++-+-+-1;"
assert 10 "-10+20;"
assert 10 "- -10;"
assert 10 "- - +10;"

assert 1 "1==1;"
assert 0 "1==2;"
assert 1 "1!=2;"
assert 0 "1!=1;"
assert 1 "1<2;"
assert 0 "1<1;"
assert 0 "2<1;"
assert 1 "1<=2;"
assert 1 "1<=1;"
assert 0 "2<=1;"

assert 1 "2>1;"
assert 0 "1>1;"
assert 0 "1>2;"
assert 1 "2>=1;"
assert 1 "1>=1;"
assert 0 "1>=2;"

assert 3 "1;2;3;"

assert 8 'a=3; z=5; a+z;'
assert 6 'a=b=3; a+b;'
assert 3 'foo=3; foo;'
assert 8 'foo=3; bar=5; foo+bar;'

echo OK