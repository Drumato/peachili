# Tokenizer

- 入力: ソースプログラム `String`
- 出力: トークン列 `Vec<Token>`

## 概要

```rust
fn tokenize(source: String) -> Vec<Token> {
    // トークン列の初期化
    let tokens : Vec<Token> = Default::default();

    // トークンやエラーの位置用に定義
    // 各トークンはこの情報を自身に保有する
    let mut row: usize = 1;
    let mut column: usize = 1;

    loop {
        // 一つのトークンを読み込む
        let te : Result<Token, TokenizeError> = scan(&mut source, &mut row, &mut column);

        match te {
            Ok(tok) => {
                // 空白やカンマなど，構文解析アルゴリズム上で意味を持たないトークンは追加しない
                if t.should_ignore() {
                    continue;
                }
                tokens.push(t);
            },
            Err(e) => {
                match e.kind {
                    // これ以上トークナイズできない -> EOFを挿入し終了
                    TokenizeError::EMPTY => {
                        tokens.push(
                            Token {
                                kind: EOF, 
                                pos: Default::default()
                            }
                        );
                        break;
                    }
                    // 他のエラーはコンパイルエラーとし，出力してプロセスを落とす
                    err => err.output_and_panic(),
                }
            }
        }
    }

    tokens
}
```