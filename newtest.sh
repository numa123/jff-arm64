# #!/bin/bash

# # 一時ディレクトリの作成と削除のトラップ設定
# tmp=$(mktemp -d /tmp/jff-test-XXXXXX)
# trap 'rm -rf $tmp' INT TERM HUP EXIT

# # 共通ファイル common.c を事前にコンパイルしてオブジェクトファイルを作成
# gcc -c -o "$tmp/common.o" test/common.c || exit

# # CC (コンパイラ) を指定、デフォルトでgccを使用するように設定
# CC=${CC:-gcc}

# # 対象ファイルを決定（引数がある場合はそのファイルのみ、ない場合は test/*.c 全部）
# if [[ -n "$1" ]]; then
#   # 引数がある場合、指定されたファイルを対象に
#   src_files="test/$1"
# else
#   # 引数がない場合、全ての *.c を対象に
#   src_files=test/*.c
# fi

# # 対象ファイルをループで処理
# for src_file in $src_files; do
#   # common.c はスキップ
#   if [[ "$src_file" == "test/common.c" ]]; then
#     continue
#   fi
  
#   # ファイル名から拡張子を除いたものをベース名とする
#   base_name=$(basename "$src_file" .c)

#   # プリプロセッサでインクルードを展開
#   $CC -E -P "$src_file" -o "$tmp/${base_name}_preprocessed.c" || exit

#   # 自作コンパイラでアセンブリコードを生成
#   ./target/debug/jff "$tmp/${base_name}_preprocessed.c" > "$tmp/${base_name}.s" || exit

#   # システムコンパイラでリンクして実行ファイルを作成
#   $CC -o "$tmp/$base_name" "$tmp/${base_name}.s" "$tmp/common.o" || exit

#   # 実行して結果を確認
#   echo "Running $base_name"
#   "$tmp/$base_name" || exit 1
#   echo
# done

# echo "All tests passed."
#!/bin/bash

# 一時ディレクトリの作成と削除のトラップ設定
tmp=$(mktemp -d /tmp/jff-test-XXXXXX)
trap 'rm -rf $tmp' INT TERM HUP EXIT

# 共通ファイル common.c を事前にコンパイルしてオブジェクトファイルを作成
gcc -c -o "$tmp/common.o" test/common.c || exit

cargo build

# CC (コンパイラ) を指定、デフォルトでgccを使用するように設定
CC=${CC:-gcc}

# 対象ファイルを決定（引数がある場合はそのファイルのみ、ない場合は test/*.c 全部）
if [[ -n "$1" ]]; then
  # 引数がある場合、指定されたファイルを対象に
  src_files="test/$1"
else
  # 引数がない場合、全ての *.c を対象に
  src_files=test/*.c
fi

# 対象ファイルをループで処理
for src_file in $src_files; do
  # common.c はスキップ
  if [[ "$src_file" == "test/common.c" ]]; then
    continue
  fi
  
  # ファイル名から拡張子を除いたものをベース名とする
  base_name=$(basename "$src_file" .c)

  # プリプロセッサでインクルードを展開
  $CC -E -P "$src_file" -o "$tmp/${base_name}_preprocessed.c" || exit

  # 自作コンパイラでアセンブリコードを生成
  ./target/debug/jff "$tmp/${base_name}_preprocessed.c" > "$tmp/${base_name}.s" || exit

  # システムコンパイラでリンクして実行ファイルを作成
  $CC -o "$tmp/$base_name" "$tmp/${base_name}.s" "$tmp/common.o" || exit


  # 実行して結果を確認
  echo "Running $base_name"
  if ! "$tmp/$base_name"; then
    echo "Test failed for $base_name"
    # echo "=== Assembly output ==="
    # cat "$tmp/${base_name}.s"  # .sファイルの内容を出力
    exit 1
  fi
  echo
done

echo "All tests passed."


# ./newtest.sh [filename]で、指定したファイルのみをテストできる