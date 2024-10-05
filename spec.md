## 前回の反省点
- tokenの受け渡しを構造体経由にしたらもっと良さそう
- enumとstructをうまく使って、冗長なメンバを持たないようにして、unwrapを減らす
- 変数のオフセット計算をしっかりして、多次元配列もきちんと実装する
- intの方が良い気がしなくもないけど一旦longのままでやる
## EBNF
- expr = equality
- equality = add ("==" add | "!=" add)*
- add = mul ("+" mul | "-" mul)*
- mul = unary ("*" unary | "/" unary)*
- unary = ("+" | "-") unary | primary
- primary = num | "(" expr ")"

## 現在サポート中の演算子の優先順位
低
1. ==, !=
3. +, -
4. *, /
5. 単項+, 単項-
6. ()

## 演算子の優先順位
![alt text](operator-priority.png)
source: https://c-lang.sevendays-study.com/appendix4.html

