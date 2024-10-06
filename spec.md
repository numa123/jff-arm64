## 前回の反省点
- tokenの受け渡しを構造体経由にしたらもっと良さそう
- enumとstructをうまく使って、冗長なメンバを持たないようにして、unwrapを減らす
- 変数のオフセット計算をしっかりして、多次元配列もきちんと実装する
- intの方が良い気がしなくもないけど一旦longのままでやる
- 型の追加は以下の2パターンが考えられる
  - Optionを使って、後で再帰的に型をつける <- 今回はこっち(chibiccも)
  - Nodeを作成する際にtyをつけて、Nodeを生成する際に毎回tyをつける。

## EBNF
- declspec = "int"
- decltype = "*"* 
  - 入力: declaration_specifierとtoken。出力: type
- declaration = declspec ( decltype ident ("=" expr)? ("," decltype ("=" expr)?)* )? ";"

- program = stmt*
  - 気持ちとしては、今の段階のプログラムは、stmtが0個以上あるものとしてコンパイラを作成している、というものだと思う
- stmt = "return" expr ";" | expr-stmt | "{" compound-stmt | "if" "(" expr ")" stmt ("else" stmt)? | "for" "(" expr-stmt expr? ";" expr? ")" stmt | "while" "(" expr ")" stmt
- compound-stmt = (declaration | stmt)* "}"
- expr-stmt = expr? ";"
- expr = assign
- assign = equality ("=" assign)?
- equality = relational ("==" relational | "!=" relational)*
- relational = add ("<" add | "<=" add | ">" add | ">=" add)*
- add = mul ("+" mul | "-" mul)*
- mul = unary ("*" unary | "/" unary)*
- unary = ("+" | "-" | "*" | "&") unary | primary
- primary = num | "(" expr ")" | ident args?
- args = "(" ")"

## 現在サポート中の演算子の優先順位
低
1. ==, !=
2. <, <=, >, >=
3. +, -
4. *, /
5. 単項+, 単項-, 単項*, 単項&
6. ()

## 演算子の優先順位
![alt text](operator-priority.png)
source: https://c-lang.sevendays-study.com/appendix4.html

# メモ
- 変数を定義するところあたりから、気合い入れて設計していくべき
- ポインタ同士の掛け算とかはどういう扱いになっているんだ今のコードだと

- multiletter variableを実装している時のFunctionの気持ちとしては、test.shで書いているシングルクォーテーションで囲まれた部分は、関数のブロック内で起こっていることみたいな感覚。int main() { ここ } の、ここの部分。
- そろそろコードが冗長になってきた
- 型のつけ忘れとかできちんと動かない場合がある。
- 冗長なのにわかりづらいコードになってきた。


# 注意
```
                    let var = Var {
                        name: name.clone(),
                        offset: self.variables.len() as isize * 8, // 一旦今はlongだけのサポート。でもここは熟考する。usizeをisizeにしているから、大きすぎる値だとおかしくなりそう
                    };
```
- Varのアドレスはこれのままで良いのか？？