#!/bin/bash
cat <<EOF | gcc -xc -c -o tmp2.o -
int ret3() { return 3; }
int ret5() { return 5; }
int add(int x, int y) { return x+y; }
int sub(int x, int y) { return x-y; }
int add6(int a, int b, int c, int d, int e, int f) {
    return a+b+c+d+e+f;
}
int add8(int a, int b, int c, int d, int e, int f, int g, int h) {
    return a+b+c+d+e+f+g+h;
}
EOF

assert() {
	expected="$1"
	input="$2"
	./target/debug/jff "$input" > tmp.s || exit
	gcc-14 -o tmp tmp.s tmp2.o
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


# ポインタからchar, short, int, longへのキャスト
assert 108 'int main() { int x = 1; (char)(int *)&x; }'
assert 1 'int main() { int x = 1; (short)(int *)&x; }'
assert 1 'int main() { int x = 1; (int)(int *)&x; }'
assert 1 'int main() { int x = 1; (long)(int *)&x; }'

# char, short, int, longからポインタへのキャスト
assert 5 'int main() { char x = 5; (int *)&x; *(char *)&x; }'
assert 10 'int main() { short x = 10; (int *)&x; *(short *)&x; }'
assert 100 'int main() { int x = 100; (int *)&x; *(int *)&x; }'
assert 1000 'int main() { long x = 1000; (int *)&x; *(long *)&x; }'

# charから他の型へのキャストテスト
# assert 3 'int main() { char x = 3; return (int)x; }'
# assert 3 'int main() { char x = 3; return (short)x; }'
# assert 3 'int main() { char x = 3; return (long)x; }'

# shortから他の型へのキャストテスト
# assert 42 'int main() { short x = 42; return (char)x; }'
# assert 42 'int main() { short x = 42; return (int)x; }'
# assert 42 'int main() { short x = 42; return (long)x; }'

# # intから他の型へのキャストテスト
# assert 100 'int main() { int x = 100; return (char)x; }'
# assert 100 'int main() { int x = 100; return (short)x; }'
# assert 100 'int main() { int x = 100; return (long)x; }'

# # longから他の型へのキャストテスト
# assert 127 'int main() { long x = 127; return (char)x; }'
# assert 32767 'int main() { long x = 32767; return (short)x; }'
# assert 2147483647 'int main() { long x = 2147483647; return (int)x; }'

# # 特定のビット境界を超えるテストケース（符号付きキャスト）
# assert -128 'int main() { char x = -128; return (int)x; }'
# assert -1 'int main() { short x = -1; return (char)x; }'
# assert -32768 'int main() { int x = -32768; return (short)x; }'
# assert -2147483648 'int main() { long x = -2147483648; return (int)x; }'

echo OK