# jffの仕様
コミット時点でのコンパイラもとい言語の仕様(EBNF等)を記述していく
ただここの変更忘れが多いので、これが各コミット時の仕様に必ずしもなっていない点に注意。

## コンパイルできるもの
- 数字
- 加減算
- 四則演算(+, -, *, /, ())
- 比較演算子(==, !=, <, <=, >, >=)
- 一文字以上のローカル変数宣言。グローバル変数を使っていない影響で、冗長なコードが多くなっている。そろそろ構造体をいい感じに使うと良いのかもしれない。offsetの計算にunsafeを使ってしまっている。変数名に使用できるものは最初が[a-zA-Z_]、2文字目以降が[a-zA-Z_1-9]
- return
- {}
- if
- for
- while
- 引数なし関数呼び出し
- 引数あり関数呼び出し(8個まで)
- intから始まる変数定義
  - "int" を使えているというか、intを無視しているという形になっているのは良くない点かもしれない。
  - int x, y; は未対応
- 関数宣言
  - 方針
    - 今codegenのトップレベルはgen_stmtだけど、それをgen_funcみたいな感じにして、関数から始まるようにする。
    - 関数の構造体の中に、ローカル変数のベクタ, bodyに、stmt*
    - 単に、今までのブロックにint main() がついただけのものをコンパイルできるようにするか
  - 実装方針
    - VARIABLESをf.variablesに変更。VARIABLESを残しておけばグローバル変数の定義に使えそうだけど、それはまたその時にするから今は消そう

## 演算の優先順位(低い順)
低
1. =
2. ==, !=
3. < <= > >=
4. +, -
5. *, /
6. 単項+, 単項-
7. ()

高

## 大きな課題
- ~~現在プロローグや、エピローグがなくても関数呼び出しがないからか、テストは通る。でも関数を呼び出していく中でsp, lp, fpはきちんとやらないといけない。~~
  - まだ実装してないけど別ブランチで呼べることを確認済み(引数なしに限っていて、引数ありは未確認)

## EBNF
- function = "int" ident "(" ")" compound-stmt
- declaration = "int" expr-stmt

- stmt = expr-stmt | "return" expr ";" | "{" compound-stmt | "if" "(" expr ")" stmt ("else" stmt)? | "for" "(" expr_stmt expr? ";" expr? ")" stmt | "while" "(" expr ")" stmt
- compound-stmt = (declaration | stmt)* "}" // "{" があるかどうかでexpr-stmtと区別している
- expr_stmt = expr? ";"
- expr = assign
- assign = equality ("=" equality)?
- equality = relational ("==" relational | "!=" relational)*
- relational = add ("<" add | "<=" add | ">" add | ">=" add)*
- add = mul ("+" mul | "-" mul)*
- mul = unary ( "\*" unary | "/" unary)*
- unary = ("+" | "-")? primary
- primary = num | ident args? | "(" expr ")"
- args = "(" ((expr ",")* expr)? ")"

### argsのパース
1. exprがある場合、2に。なければなし
2. ,がある場合、,を消費してvec argに入れる。2に戻る。なければ3に
3. exprを入れて、終わり

- ndfunccallだったら、argsの左からx0, x1, ... x7みたいな感じか。


## 謎
- str wzrは、なんのためにやっているのか、やっていない時もあるし。ABIに書いてある？

## 課題
- a=1; a; っていう記述は許さない方が良いのか？
- 

## メモ
アラインメント、ゼロ徐算, removeでの消費、あたりの自信が少しないかも。
特に16バイトで整理しているけど、それはabi的に大丈夫なのか？printfを呼べるのか？という懸念がある
~~as_str()とto_string()の違い~~ Stringに変換するか&strに変換するかの違い

やたらclone()してる気がする、大丈夫か？
あとStringばっかり使っている気がする
↑ あとで必要に応じて最適化する。今は作りやすさ、わかりやすさ優先


入力を信頼しすぎている気がする(コンパイルすべきでない記述を弾けているかはあまり確認していない)
ここでは正常系が動く方が何倍も重要と考えることにして、一旦このままやってみる

chibiccでは、gen_exprの中で、rhsから処理してる。そっちの方がわかりやすいのかと思ったけど、どっちにしろ少し混乱するので、コメントで補足


tokens[0]が現在処理しているトークンってわかりづらいかな


```
     Running `target/debug/jff 'abc=1;'`
[Token { kind: TkIdent, val: 0, str: "abc", loc: 3 }, Token { kind: TkPunct, val: 0, str: "=", loc: 3 }, Token { kind: TkNum, val: 1, str: "1", loc: 4 }, Token { kind: TkPunct, val: 0, str: ";", loc: 5 }]
.global _main
```
このabcのlocが3なの気になる、良いんだっけ

~~ローカル変数用の領域確保が、グローバル変数のVARIABLESなの絶対良くない。そろそろリファクタリングした方が良い？構造体に入れるなど。~~ 絶対良くないとは言い切れない気がする。並列処理はまだ未サポートだから。ただ今後サポートしたいからその時は考慮する

NdBlock無しでもいけるかなって思ったけど、stmtを中で複数回実行しないといけないから、NodeとNdBlockを入れた

{int a = 1; a;} というプログラムが実行できるのは良くないのかもしれない？

なんでBoxって、ポインタより安全なんだ?

BCOUNTを最後にインクリメントするのはどうなのか。
exprとかは暗黙的にtokens.removeしてくれてるけどそれでよいんだかな、コード書く時に紛らわしくないかな

今はブロックじゃなくてもコンパイルできちゃうけど、普通はダメなのかな？あと変数名もちゃんとしないと。
エラーの時、eprintlnだけじゃ止まらないの、ダメかも？ -> panic!();入れておく

## 過去に発生したバグの解決ログ
```
なんでunaryが2回呼ばれるはずが3回も呼ばれているのか
問題は、なぜtokensがからなのに、unary, mul, add, relational, equality, assign, exprが呼ばれているのか
exprが呼ばれるのはどのタイミングか(tokensがから), expr-stmtが呼ばれている。どうして
returnわすれ！！！
```

## 注意
- 今、数値は8バイト使っていて、long型になっている。intにした場合は4バイトにするべき。
- いろいろバイト数をハードコードしている部分はテキトーになっている恐れあり。
- プロローグのstr wzrとかが必要な場合とかに注意
- gotboltの`armv8-a clang 18.1.0`と、手元でコンパイルした場合はちょっと違うから注意。gotboltはmainだけど、手元だと_mainになるとか。関数名は_から始まるっぽいとか。そろそろABIとか読もうか
- spは16バイトアラインメントっぽい
- LP64とやらに準拠してポインタを実装すれば良いのかな
- wzrは4バイト, xzrは16バイト
- bl命令をすると、自動的にlpが保存されて、ret   // 'LR'に保存されたアドレスに戻る

## 手元のコンパイラ
```
~/r/s/jff ❯ (feature/zero-arity-function-calls) clang --version                                                                (base) 
Homebrew clang version 18.1.8
Target: arm64-apple-darwin23.5.0
Thread model: posix
InstalledDir: /opt/homebrew/opt/llvm/bin
```

## arm64のPCS(Procedure Call Standard)
```
5.7   Pointers
Code and data pointers are either 64-bit or 32-bit unsigned types [5]. A NULL pointer is always represented by all-bits-zero.

All 64 bits in a 64-bit pointer are always significant. When tagged addressing is enabled, a tag is part of a pointer’s value for the purposes of pointer arithmetic. The result of subtracting or comparing two pointers with different tags is unspecified. See also Memory addresses, below. A 32-bit pointer does not support tagged addressing.

Note

(Beta)

The A64 load and store instructions always use the full 64-bit base register and perform a 64-bit address calculation. Care must be taken within ILP32 to ensure that the upper 32 bits of a base register are zero and 32-bit register offsets are sign-extended to 64 bits (immediate offsets are implicitly extended).

```

![alt text](general_register.png)

## 参考にするコンパイラを、clangからgccにする
llvmじゃない文なんか良い
