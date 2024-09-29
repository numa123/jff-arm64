# jff(aarch64向けのCコンパイラ)の仕様
開発の都度、その時点でのコンパイラもとい言語の仕様(EBNF等)を記述していく

## コンパイルできるもの
- 数字1つ
- 加減算
- 四則演算(*, /, ())
- 比較演算子(==, !=, <, <=, >, >=)
- 一文字のローカル変数宣言
- 一文字以上のローカル変数宣言。グローバル変数を使っていない影響で、冗長なコードが多くなっている。そろそろ構造体をいい感じに使うと良いのかもしれない。offsetの計算にunsafeを使ってしまっている。変数名に使用できるものはa-zA-Z
- return 式 ;

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

## EBNF
- stmt = expr-stmt | "return" expr ";" | "{" compound-stmt | "if" "(" expr ")" stmt ("else" stmt)?
- compound-stmt = stmt* "}" // "{" があるかどうかでexpr-stmtと区別している
- expr_stmt = expr? ";"
- expr = assign
- assign = equality ("=" equality)?
- equality = relational ("==" relational | "!=" relational)*
- relational = add ("<" add | "<=" add | ">" add | ">=" add)*
- add = mul ("+" mul | "-" mul)*
- mul = unary ( "\*" unary | "/" unary)*
- unary = ("+" | "-")? primary
- primary = num | ident | "(" expr ")"

## 課題
- tokenizeを別のファイルに切り分けないと
- a=1; a; っていう記述は許さない方が良いのか？

## メモ
数字一つは受け取れた。次は1+1。
1+1も行けた。次は2-1, 5+20-4
mulを追加して、数字のみは受け取れた、次は1+1, 2-1, その後に2*2など
掛け算と割り算ができた、ただ割り算の分母が0かどうかを確かめれてない。そういう算術命令ありそう
chibiccを参考にしながらやるとすんなりできた


アラインメント、ゼロ徐算, removeでの消費、あたりの自信が少しないかも。
特に16バイトで整理しているけど、それはabi的に大丈夫なのか？printfを呼べるのか？という懸念がある
as_str()とto_string()の違い
やたらclone()してる気がする、大丈夫か？
あとStringばっかり使っている気がする


入力を信頼しすぎているのかな？まあここでは正常系が動く方が何倍も重要だから、一旦このままやってみる


関数gから呼び出された関数fのスタックフレームにあるものは上から
- fのリターンアドレス。fを呼び出したコードの次？ lp(x30)
- fの呼び出し時点のベースポインタ(スタックフレームの基準) fp(x29)
- 変数たち
- (関数終わり)
- fpをgのfpに戻す
- spを戻す
- lpに帰る

gotboltによると
- プロローグ
```
  sub sp, sp, #32
  stp x29, x30, [sp, #16]
  add x29, sp, #16
```
- エピローグ
```
  ldp x29, x30, [sp, #16]
  add sp, sp, #32
  ret
```


chibiccでは、gen_exprの中で、rhsから処理してる。そっちの方がわかりやすいかも？


複数文字のローカル変数のサポートに必要なもの
ローカル変数: 名前とオフセットに使える一位の数値


tokens[0]が現在処理しているトークンってわかりづらいかな


     Running `target/debug/jff 'abc=1;'`
[Token { kind: TkIdent, val: 0, str: "abc", loc: 3 }, Token { kind: TkPunct, val: 0, str: "=", loc: 3 }, Token { kind: TkNum, val: 1, str: "1", loc: 4 }, Token { kind: TkPunct, val: 0, str: ";", loc: 5 }]
.global _main
このabcのlocが3なの気になるな

ローカル変数用の領域確保が、グローバル変数のVARIABLESなの絶対良くない。そろそろリファクタリングした方が良い？構造体に入れるなど。

NdBlock無しでもいけるかなって思ったけど、stmtを中で複数回実行しないといけないから、NodeとNdBlockを入れた

{int a = 1; a;} というプログラムが実行できるのは良くないのかもしれない？

なんでBoxって、ポインタより安全なんだ?

## バグ
なんでunaryが2回呼ばれるはずが3回も呼ばれているのか
問題は、なぜtokensがからなのに、unary, mul, add, relational, equality, assign, exprが呼ばれているのか
exprが呼ばれるのはどのタイミングか(tokensがから), expr-stmtが呼ばれている。どうして
returnわすれ！！！