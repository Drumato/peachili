use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use crate::compiler::common::frontend::typed_ast as ast;
use crate::compiler::common::frontend::{frame_object, peachili_type};

const PUSH_RAX: &'static str = "  push rax\n";

pub fn codegen_main(ast_roots: VecDeque<ast::Root>) -> String {
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

    ".intel_syntax noprefix\n".to_owned() + &assembly_file
}

fn gen_root(
    ast_root: &ast::Root,
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
    func: &ast::Function,
    mut str_id_set: HashSet<(u64, String)>,
) -> (String, HashSet<(u64, String)>) {
    let mut fn_str = format!(".global \"{}\"\n", func.name.trim_start_matches("main::"));
    fn_str += &format!("\"{}\":\n", func.name.trim_start_matches("main::"));
    fn_str += &gen_fn_prologue(func.stack_size);

    for (i, (param_name, _param_ty)) in func.params.iter().enumerate() {
        let param = func.local_variables.get(param_name).unwrap();

        let param_reg = param_reg64(i);
        fn_str += &format!("  mov -{}[rbp], {}\n", param.stack_offset, param_reg);
    }

    for st in func.stmts.iter() {
        let (stmt_str, set) = gen_stmt(st, &func.local_variables, str_id_set);
        fn_str += &stmt_str;
        str_id_set = set;
    }
    (fn_str, str_id_set)
}

fn gen_fn_prologue(stack_size: usize) -> String {
    let mut s = format!("  push rbp\n");
    s += "  mov rbp, rsp\n";
    s += &format!("  sub rsp, {}\n", stack_size);
    s
}

fn gen_stmt(
    st: &ast::Statement,
    local_variables: &HashMap<String, frame_object::FrameObject>,
    str_id_set: HashSet<(u64, String)>,
) -> (String, HashSet<(u64, String)>) {
    match st {
        ast::Statement::Nop => (String::new(), str_id_set),
        ast::Statement::Expr { expr } => gen_code(
            "Expression Statement",
            |set| {
                let (ex_str, set2) = gen_expr(expr, local_variables, set);
                (ex_str, set2)
            },
            str_id_set,
        ),
        ast::Statement::Asm { insts } => gen_code(
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
    ex: &ast::Expression,
    local_variables: &HashMap<String, frame_object::FrameObject>,
    mut str_id_set: HashSet<(u64, String)>,
) -> (String, HashSet<(u64, String)>) {
    match &ex.kind {
        ast::ExprKind::Assignment {
            var_name: _,
            var_stack_offset,
            expr,
        } => gen_code(
            "Assignment",
            |set| {
                let mut s = gen_lvalue_with(*var_stack_offset);
                s += PUSH_RAX;
                let (expr_s, set) = gen_expr(&expr.as_ref().borrow(), local_variables, set);
                s += &expr_s;
                s += &gen_store(&ex.ty);
                (s, set)
            },
            str_id_set,
        ),
        ast::ExprKind::Multiplication { lhs, rhs }
        | ast::ExprKind::Division { lhs, rhs }
        | ast::ExprKind::Addition { lhs, rhs }
        | ast::ExprKind::Subtraction { lhs, rhs } => {
            gen_binary_expr(ex, lhs, rhs, local_variables, str_id_set)
        }
        ast::ExprKind::Integer { value } => gen_code(
            "Integer Literal",
            |set| (format!("  mov rax, {}\n", value), set),
            str_id_set,
        ),
        ast::ExprKind::UnsignedInteger { value } => gen_code(
            "Unsigned Integer Literal",
            |set| (format!("  mov rax, {}\n", value), set),
            str_id_set,
        ),
        ast::ExprKind::Identifier {
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
        ast::ExprKind::Negative { child } => gen_code(
            "Negative Expression",
            |set| {
                let (child_str, set) = gen_expr(&child.as_ref().borrow(), local_variables, set);
                let mut s = child_str;
                s += &format!("  neg rax\n");
                (s, set)
            },
            str_id_set.clone(),
        ),
        ast::ExprKind::Call { ident, params } => gen_code(
            "Call Expression",
            |mut set| {
                let mut s = String::new();
                for param in params.iter() {
                    let (param_str, set2) = gen_expr(param, local_variables, set);
                    set = set2;

                    s += &param_str;
                    s += PUSH_RAX;
                }

                let nparams = params.len();

                for i in 0..nparams {
                    let param_reg = param_reg64(nparams - i - 1);
                    s += &format!("  pop {}\n", param_reg);
                }

                s += &format!("  call \"{}\"\n", ident.trim_start_matches("main::"));
                (s, set)
            },
            str_id_set,
        ),
        ast::ExprKind::StringLiteral { contents, id } => {
            str_id_set.insert((*id, contents.clone()));
            gen_code(
                "String Literal",
                |set| (format!("  lea rax, .str{}[rip]\n", id), set),
                str_id_set,
            )
        }
        ast::ExprKind::True => gen_code(
            "True Literal",
            |set| ("  mov rax, 1\n".to_string(), set),
            str_id_set,
        ),
        ast::ExprKind::False => gen_code(
            "True Literal",
            |set| ("  mov rax, 0\n".to_string(), set),
            str_id_set,
        ),
    }
}

fn gen_store(_ty: &peachili_type::PeachiliType) -> String {
    let mut s = "  pop rdi\n".to_string();

    s += "  mov [rdi], rax\n";
    s
}

fn gen_binary_expr(
    ex: &ast::Expression,
    lhs: &Rc<RefCell<ast::Expression>>,
    rhs: &Rc<RefCell<ast::Expression>>,
    local_variables: &HashMap<String, frame_object::FrameObject>,
    str_id_set: HashSet<(u64, String)>,
) -> (String, HashSet<(u64, String)>) {
    // rhsのコンパイル結果(rax)をスタックに保持しておくことで，
    // lhs -> rax; rhs -> rdiを実現
    let (mut s, set) = gen_expr(&rhs.as_ref().borrow(), local_variables, str_id_set);
    s += PUSH_RAX;
    let (s2, set) = gen_expr(&lhs.as_ref().borrow(), local_variables, set);
    s += &s2;
    s += "  pop rdi\n";

    match &ex.kind {
        ast::ExprKind::Addition { lhs: _, rhs: _ } => {
            s += "  add rax, rdi\n";
        }
        ast::ExprKind::Subtraction { lhs: _, rhs: _ } => {
            s += "  sub rax, rdi\n";
        }
        ast::ExprKind::Multiplication { lhs: _, rhs: _ } => {
            s += "  imul rax, rdi\n";
        }
        ast::ExprKind::Division { lhs: _, rhs: _ } => {
            s += "  cqo\n";
            s += "  idiv rdi\n";
        }
        _ => unreachable!(),
    }

    (s, set)
}
fn gen_lvalue(ex: &ast::Expression) -> String {
    match ex.kind {
        ast::ExprKind::Identifier {
            list: _,
            stack_offset,
        } => gen_lvalue_with(stack_offset),
        _ => panic!("cannot generate code {:?} as lvalue", ex),
    }
}
fn gen_lvalue_with(stack_offset: usize) -> String {
    format!("  lea rax, -{}[rbp]\n", stack_offset)
}
fn gen_load(ty: &peachili_type::PeachiliType) -> String {
    match ty.size {
        1 => format!("  movsx rax, BYTE PTR [rax]\n"),
        2 => format!("  movswq rax, WORD PTR [rax]\n"),
        4 => format!("  movsxd rax, DWORD PTR [rax]\n"),
        _ => format!("  mov rax, [rax]\n"),
    }
}

fn param_reg64(i: usize) -> &'static str {
    match i {
        0 => "rdi",
        1 => "rsi",
        2 => "rdx",
        3 => "rcx",
        4 => "r8",
        5 => "r9",
        _ => panic!("parameter register exhausted"),
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
