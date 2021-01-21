use std::collections::{HashMap, HashSet};

use crate::compiler::arch::aarch64;

pub fn codegen_main(ast_roots: Vec<aarch64::Root>) -> String {
    let mut str_id_set: HashSet<(u64, String)> = Default::default();

    let assembly_file = ast_roots
        .iter()
        .map(|root| {
            let (root_str, set) = gen_root(root, str_id_set.clone());
            str_id_set = set;
            root_str
        })
        .collect::<Vec<String>>()
        .join("# translation unit's end\n");

    assembly_file
}

fn gen_root(
    ast_root: &aarch64::Root,
    mut str_id_set: HashSet<(u64, String)>,
) -> (String, HashSet<(u64, String)>) {
    let translation_unit_str = ast_root
        .functions
        .iter()
        .map(|func| {
            let (fn_str, set) = gen_fn(func, str_id_set.clone());
            str_id_set = set;
            fn_str
        })
        .collect::<Vec<String>>()
        .join("\n");

    (translation_unit_str, str_id_set)
}

fn gen_fn(
    func: &aarch64::Function,
    mut str_id_set: HashSet<(u64, String)>,
) -> (String, HashSet<(u64, String)>) {
    let mut fn_str = format!(".global \"{}\"\n", func.name.trim_start_matches("main::"));
    fn_str += &format!("\"{}\":\n", func.name.trim_start_matches("main::"));
    fn_str += &gen_fn_prologue(func.stack_size);

    for (i, (param_name, _param_ty)) in func.params.iter().enumerate() {
        let param = func.local_variables.get(param_name).unwrap();

        fn_str += &format!("  str x{}, [sp, #{}]\n", i, param.stack_offset);
    }

    for st in func.stmts.iter() {
        let (stmt_str, set) = gen_stmt(st, &func.local_variables, str_id_set);
        fn_str += &stmt_str;
        str_id_set = set;
    }

    fn_str += &gen_fn_epilogue(func.stack_size);

    (fn_str, str_id_set)
}

fn gen_fn_prologue(stack_size: usize) -> String {
    // リンクレジスタ，フレームポインタのために16バイト多くアロケートする
    let mut s = format!("  sub sp, sp, #{}\n", stack_size + 16);
    s += &format!("  stp x29, x30, [sp, #{}]\n", stack_size);
    s += "  mov x29, sp\n";
    s
}

fn gen_fn_epilogue(stack_size: usize) -> String {
    let s = format!("  add sp, sp, #{}\n", stack_size + 16);
    s
}

fn gen_stmt(
    st: &aarch64::Statement,
    local_variables: &HashMap<String, aarch64::FrameObject>,
    str_id_set: HashSet<(u64, String)>,
) -> (String, HashSet<(u64, String)>) {
    match st {
        aarch64::Statement::Expr { expr } => gen_code(
            "Expression Statement",
            |set| {
                let (ex_str, set2) = gen_expr(expr, local_variables, set);
                (ex_str, set2)
            },
            str_id_set,
        ),
        aarch64::Statement::Asm { insts } => gen_code(
            "Asm Statement",
            |set| {
                (
                    insts
                        .iter()
                        .map(|inst| format!("  {}\n", inst))
                        .collect::<Vec<String>>()
                        .join(""),
                    set,
                )
            },
            str_id_set,
        ),
    }
}

fn gen_expr(
    ex: &aarch64::Expression,
    local_variables: &HashMap<String, aarch64::FrameObject>,
    mut str_id_set: HashSet<(u64, String)>,
) -> (String, HashSet<(u64, String)>) {
    match &ex.kind {
        aarch64::ExprKind::Integer { value } => gen_code(
            "Integer Literal",
            |set| (format!("  mov x0, #{}\n", value), set),
            str_id_set,
        ),
        aarch64::ExprKind::UnsignedInteger { value } => gen_code(
            "Unsigned Integer Literal",
            |set| (format!("  mov x0, #{}\n", value), set),
            str_id_set,
        ),
        aarch64::ExprKind::Identifier {
            list: _,
            stack_offset: _,
        } => gen_code(
            "Identifier",
            |set| {
                let mut s = gen_lvalue(ex);
                s += &gen_load(&ex.ty);
                (s, set)
            },
            str_id_set,
        ),
        aarch64::ExprKind::Negative { child } => gen_code(
            "Negative Expression",
            |set| {
                let (child_str, set) = gen_expr(&child.as_ref().borrow(), local_variables, set);
                let mut s = child_str;
                s += &format!("  subs x0, xzr, x0\n");
                (s, set)
            },
            str_id_set.clone(),
        ),
        aarch64::ExprKind::Call { ident, params } => gen_code(
            "Call Expression",
            |mut set| {
                let mut s = String::new();
                for (i, param) in params.iter().enumerate() {
                    let (param_str, set2) = gen_expr(param, local_variables, set);
                    set = set2;

                    s += &param_str;
                    s += &format!("  mov x{}, x0\n", params.len() - i);
                }

                s += &format!("  bl \"{}\"\n", ident.trim_start_matches("main::"));
                (s, set)
            },
            str_id_set,
        ),
        aarch64::ExprKind::StringLiteral { contents, id } => {
            str_id_set.insert((*id, contents.clone()));
            gen_code(
                "String Literal",
                |set| (format!("  adrp x0, .str{}[rip]\n", id), set),
                str_id_set,
            )
        }
        aarch64::ExprKind::True => gen_code(
            "True Literal",
            |set| ("  mov x0, #1\n".to_string(), set),
            str_id_set,
        ),
        aarch64::ExprKind::False => gen_code(
            "True Literal",
            |set| ("  mov x0, #0\n".to_string(), set),
            str_id_set,
        ),
    }
}
fn gen_lvalue(ex: &aarch64::Expression) -> String {
    match ex.kind {
        aarch64::ExprKind::Identifier {
            list: _,
            stack_offset,
        } => format!("  adrp x0, [sp, #-{}]\n", stack_offset),
        _ => panic!("cannot generate code {:?} as lvalue", ex),
    }
}

fn gen_load(ty: &aarch64::PeachiliType) -> String {
    match ty.size() {
        1 => format!("  ldr x0, [x0, #0]\n"),
        2 => format!("  ldr x0, [x0, #0]\n"),
        4 => format!("  ldr x0, [x0, #0]\n"),
        _ => format!("  ldr x0, [x0, #0]\n"),
    }
}

fn gen_code<F>(
    element_name: &str,
    mut inner: F,
    str_id_set: HashSet<(u64, String)>,
) -> (String, HashSet<(u64, String)>)
where
    F: FnMut(HashSet<(u64, String)>) -> (String, HashSet<(u64, String)>),
{
    let mut s = String::new();
    s += &format!("  # START: {}\n", element_name);
    let (inner_str, set) = inner(str_id_set);
    s += &inner_str;
    s += &format!("  # END: {}\n", element_name);

    (s, set)
}
