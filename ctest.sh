#!/bin/bash

tmp=$(mktemp -d /tmp/jff-test-XXXXXX)
trap 'rm -rf $tmp' INT TERM HUP EXIT

gcc -c -o "$tmp/common.o" test/common.c || exit

cargo build -q

CC=${CC:-gcc}

# 対象ファイルを決定（引数がある場合はそのファイルのみ、ない場合は test/*.c 全部）
if [[ -n "$1" ]]; then
  src_files="test/$1"
else
  src_files=test/*.c
fi

for src_file in $src_files; do
  if [[ "$src_file" == "test/common.c" ]]; then
    continue
  fi
  
  base_name=$(basename "$src_file" .c)

  $CC -E -P "$src_file" -o "$tmp/${base_name}_preprocessed.c" || exit

  ./target/debug/jff "$tmp/${base_name}_preprocessed.c" > "$tmp/${base_name}.s" || exit

  $CC -o "$tmp/$base_name" "$tmp/${base_name}.s" "$tmp/common.o" || exit
    # cat "$tmp/${base_name}.s"

  echo "Running $base_name"
  if ! "$tmp/$base_name"; then
    echo "Test failed for $base_name"
    # cat "$tmp/${base_name}.s" # エラー時にアセンブリを出力したい場合
    exit 1
  fi
  echo
done

echo "All tests passed."