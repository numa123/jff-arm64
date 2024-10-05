## 前回の反省点
- tokenの受け渡しを構造体経由にしたらもっと良さそう
- enumとstructをうまく使って、冗長なメンバを持たないようにして、unwrapを減らす
- 変数のオフセット計算をしっかりして、多次元配列もきちんと実装する
- intの方が良い気がしなくもないけど一旦longのままでやる

## EBNF
- program = stmt*
  - 気持ちとしては、今の段階のプログラムは、stmtが0個以上あるものとしてコンパイラを作成している、というものだと思う
- stmt = expr-stmt
- expr-stmt = expr ";"
- assign = equality
- equality = relational ("==" relational | "!=" relational)*
- relational = add ("<" add | "<=" add | ">" add | ">=" add)*
- add = mul ("+" mul | "-" mul)*
- mul = unary ("*" unary | "/" unary)*
- unary = ("+" | "-") unary | primary
- primary = num | "(" expr ")" | ident

## 現在サポート中の演算子の優先順位
低
1. ==, !=
2. <, <=, >, >=
3. +, -
4. *, /
5. 単項+, 単項-
6. ()

## 演算子の優先順位
![alt text](operator-priority.png)
source: https://c-lang.sevendays-study.com/appendix4.html