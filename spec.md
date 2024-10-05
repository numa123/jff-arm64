## 前回の反省点
- tokenの受け渡しを構造体経由にしたらもっと良さそう
- enumとstructをうまく使って、冗長なメンバを持たないようにして、unwrapを減らす
- 変数のオフセット計算をしっかりして、多次元配列もきちんと実装する
- intの方が良い気がしなくもないけど一旦longのままでやる
## EBNF
- expr = mul ("+" mul | "-" mul)*
- mul = primary ("*" primary | "/" primary)*
- primary = num | "(" expr ")"

## メモ

