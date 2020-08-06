# Parser

- 入力: トークン列 `Vec<Token>`
- 出力: モジュールレベルの構造体 `ASTRoot`

```rust
pub struct ASTRoot {
    /// 関数列
    pub funcs: Vec<function::FnId>,
    /// 構造体定義の集合
    pub typedefs: BTreeMap<String, StructDef>,
    /// 型エイリアスの集合
    pub alias: BTreeMap<String, String>,
}
```

## 概要

ほぼすべての関数が ｢トークンを受け取ってパース， パース後の **残りのトークンを返す**｣ という構造になっている．  
これはTopLevelDeclarationに限らない．  
構文については [syntax.md](../syntax.md) を参照． 

```rust
pub fn main(
    tokens: Vec<Token>,
    module_name: String
) -> ASTRoot {
    // ASTRoot構造体の定義
    let mut ast_root: ASTRoot = Default::default();

    // program -> toplevel*
    loop {
        let t = parser_util::head(&tokens);

        match t.get_kind() {
            // 関数定義のパース
            TokenKind::FUNC => {
                let (fn_id, rest_tokens) = func_def(module_name, tokens);
                tokens = rest_tokens;
                ast_root.funcs.push(fn_id);
            }
            // 構造体定義のパース
            TokenKind::STRUCT => {
                let (type_name, struct_def, rest_tokens) = struct_def(tokens);
                tokens = rest_tokens;
                ast_root.typedefs.insert(format!("{}::{}",module_name,  type_name), struct_def);
            }
            // 型エイリアスのパース
            TokenKind::PUBTYPE => {
                let (alias_name, src_name, rest_tokens) = type_alias(tokens);
                tokens = rest_tokens;
                ast_root.alias.insert(format!("{}::{}",module_name,  alias_name), src_name);
            }
            _ => break,
        }
    }

    ast_root
}
```