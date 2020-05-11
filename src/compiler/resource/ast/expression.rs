use crate::common::position as pos;
use crate::compiler::resource as res;

#[derive(Clone)]
#[allow(dead_code)]
pub struct ExpressionNode {
    pub kind: ExpressionNodeKind,
    position: pos::Position,
}

impl ExpressionNode {
    pub fn copy_ident_name(&self) -> String {
        match &self.kind {
            ExpressionNodeKind::IDENT(id_name) => IdentName::correct_name(id_name),
            _ => panic!("unexpected call `copy_ident_name` with {}", self.kind),
        }
    }
    pub fn copy_str_contents(&self) -> String {
        match &self.kind {
            ExpressionNodeKind::STRLIT(contents, _hash) => contents.to_string(),
            _ => panic!("unexpected call `copy_str_contents` with {}", self.kind),
        }
    }
    pub fn copy_pos(&self) -> pos::Position {
        let (row, col) = self.position.get_pos();
        pos::Position::new(row, col)
    }

    pub fn new_intlit(int_value: i64, ex_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::INTEGER(int_value), ex_pos)
    }

    pub fn new_strlit(contents: String, hash: u64, ex_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::STRLIT(contents, hash), ex_pos)
    }

    pub fn new_true(ex_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::TRUE, ex_pos)
    }
    pub fn new_false(ex_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::FALSE, ex_pos)
    }

    pub fn new_ident(ident: IdentName, ex_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::IDENT(ident), ex_pos)
    }

    pub fn new_call(ident: IdentName, args: Vec<ExpressionNode>, ex_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::CALL(ident, args), ex_pos)
    }

    pub fn new_neg(value: ExpressionNode, ex_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::NEG(Box::new(value)), ex_pos)
    }

    pub fn new_add(lop: ExpressionNode, rop: ExpressionNode, ex_pos: pos::Position) -> Self {
        Self::new(
            ExpressionNodeKind::ADD(Box::new(lop), Box::new(rop)),
            ex_pos,
        )
    }
    pub fn new_sub(lop: ExpressionNode, rop: ExpressionNode, ex_pos: pos::Position) -> Self {
        Self::new(
            ExpressionNodeKind::SUB(Box::new(lop), Box::new(rop)),
            ex_pos,
        )
    }
    pub fn new_mul(lop: ExpressionNode, rop: ExpressionNode, ex_pos: pos::Position) -> Self {
        Self::new(
            ExpressionNodeKind::MUL(Box::new(lop), Box::new(rop)),
            ex_pos,
        )
    }
    pub fn new_div(lop: ExpressionNode, rop: ExpressionNode, ex_pos: pos::Position) -> Self {
        Self::new(
            ExpressionNodeKind::DIV(Box::new(lop), Box::new(rop)),
            ex_pos,
        )
    }

    pub fn new_assign(lval: ExpressionNode, rval: ExpressionNode, ex_pos: pos::Position) -> Self {
        Self::new(
            ExpressionNodeKind::ASSIGN(Box::new(lval), Box::new(rval)),
            ex_pos,
        )
    }

    pub fn new_if(
        condition: ExpressionNode,
        body: Vec<res::StatementNode>,
        ex_pos: pos::Position,
    ) -> Self {
        Self::new(ExpressionNodeKind::IF(Box::new(condition), body), ex_pos)
    }

    pub fn new_ifelse(
        condition: ExpressionNode,
        body: Vec<res::StatementNode>,
        else_body: Vec<res::StatementNode>,
        ex_pos: pos::Position,
    ) -> Self {
        Self::new(
            ExpressionNodeKind::IFELSE(Box::new(condition), body, else_body),
            ex_pos,
        )
    }

    fn new(expr_kind: ExpressionNodeKind, expr_pos: pos::Position) -> Self {
        Self {
            kind: expr_kind,
            position: expr_pos,
        }
    }
}

impl std::fmt::Display for ExpressionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum ExpressionNodeKind {
    INTEGER(i64),
    STRLIT(String, u64),
    IDENT(IdentName),
    CALL(IdentName, Vec<ExpressionNode>),
    TRUE,
    FALSE,

    // unary
    NEG(Box<ExpressionNode>),

    // binary
    ADD(Box<ExpressionNode>, Box<ExpressionNode>),
    SUB(Box<ExpressionNode>, Box<ExpressionNode>),
    MUL(Box<ExpressionNode>, Box<ExpressionNode>),
    DIV(Box<ExpressionNode>, Box<ExpressionNode>),
    ASSIGN(Box<ExpressionNode>, Box<ExpressionNode>),

    // etc.
    IF(Box<ExpressionNode>, Vec<res::StatementNode>),
    IFELSE(
        Box<ExpressionNode>,
        Vec<res::StatementNode>,
        Vec<res::StatementNode>,
    ),
}

impl std::fmt::Display for ExpressionNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::TRUE => write!(f, "true"),
            Self::FALSE => write!(f, "false"),
            Self::INTEGER(v) => write!(f, "{}", v),
            Self::STRLIT(contents, _hash) => write!(f, "\"{}\"", contents),
            Self::IDENT(ident) => write!(f, "{}", ident),
            Self::CALL(ident, args) => {
                write!(f, "{}(", ident)?;

                for (i, arg) in args.iter().enumerate() {
                    if i == args.len() - 1 {
                        write!(f, "{}", arg)?;
                    } else {
                        write!(f, "{}, ", arg)?;
                    }
                }

                write!(f, ")")
            }

            // unary
            Self::NEG(v) => write!(f, "-{}", v),

            // binary
            Self::ADD(lop, rop) => write!(f, "{} + {}", lop, rop),
            Self::SUB(lop, rop) => write!(f, "{} - {}", lop, rop),
            Self::MUL(lop, rop) => write!(f, "{} * {}", lop, rop),
            Self::DIV(lop, rop) => write!(f, "{} / {}", lop, rop),

            // etc
            Self::ASSIGN(lval, rval) => write!(f, "{} = {}", lval, rval),
            Self::IF(condition, body) => {
                writeln!(f, "if ( {} ) {{ ", condition)?;

                for st in body.iter() {
                    writeln!(f, "\t\t{}", st)?;
                }

                write!(f, "}}")
            }
            Self::IFELSE(condition, body, else_body) => {
                writeln!(f, "if ( {} ) {{ ", condition)?;

                for st in body.iter() {
                    writeln!(f, "\t\t{}", st)?;
                }

                writeln!(f, "}} else {{ ")?;

                for st in else_body.iter() {
                    writeln!(f, "\t\t{}", st)?;
                }

                write!(f, "}}")
            }
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct IdentName {
    name: String,
    next: Option<Box<IdentName>>,
}

impl IdentName {
    fn new(n: String, nxt: Option<Box<IdentName>>) -> Self {
        Self { name: n, next: nxt }
    }

    pub fn new_terminated(name: String) -> Self {
        Self::new(name, None)
    }

    pub fn append_next(&mut self, next: IdentName) {
        self.next = Some(Box::new(next));
    }

    pub fn correct_name(s: &IdentName) -> String {
        let mut st = s.name.clone();

        let mut prev = &s.next;
        loop {
            if prev.is_none() {
                break;
            }
            st = format!("{}_{}", st, prev.as_ref().unwrap().name);
            prev = &prev.as_ref().unwrap().next;
        }

        st
    }

    pub fn last_name(s: &IdentName) -> String {
        let mut st = s.name.clone();

        let mut prev = &s.next;
        loop {
            if prev.is_none() {
                break;
            }
            st = prev.as_ref().unwrap().name.clone();
            prev = &prev.as_ref().unwrap().next;
        }

        st
    }
}

impl std::fmt::Display for IdentName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", IdentName::correct_name(self))
    }
}
