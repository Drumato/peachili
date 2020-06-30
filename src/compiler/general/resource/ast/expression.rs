use crate::common::position as pos;
use crate::compiler::general::resource as res;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
#[allow(dead_code)]
pub struct ExpressionNode {
    pub kind: ExpressionNodeKind,
    position: pos::Position,
}

impl ExpressionNode {
    pub fn get_ident_ids(&self) -> Vec<res::PStringId> {
        match &self.kind {
            ExpressionNodeKind::IDENT(id_name) => IdentName::correct_name(id_name),
            ExpressionNodeKind::DEREF(pointer_ex) => pointer_ex.get_ident_ids(),
            ExpressionNodeKind::MEMBER(id, _members) => id.get_ident_ids(),
            _ => panic!("unexpected call `get_ident_id` with {}", self.kind),
        }
    }

    pub fn get_str_id(&self) -> res::PStringId {
        match &self.kind {
            ExpressionNodeKind::STRLIT(contents_id, _hash) => *contents_id,
            _ => panic!("unexpected call `get_str_id` with {}", self.kind),
        }
    }
    pub fn copy_pos(&self) -> pos::Position {
        let (row, col) = self.position.get_pos();
        pos::Position::new(row, col)
    }

    pub fn operator_to_string(&self) -> String {
        match &self.kind {
            ExpressionNodeKind::ADD(_lop, _rop) => "+".to_string(),
            ExpressionNodeKind::SUB(_lop, _rop) => "-".to_string(),
            ExpressionNodeKind::MUL(_lop, _rop) => "*".to_string(),
            ExpressionNodeKind::DIV(_lop, _rop) => "/".to_string(),
            _ => panic!("{} don't have any operator.", self),
        }
    }

    pub fn new_intlit(int_value: i64, ex_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::INTEGER(int_value), ex_pos)
    }
    pub fn new_uintlit(int_value: u64, ex_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::UNSIGNEDINTEGER(int_value), ex_pos)
    }

    pub fn new_strlit(contents_id: res::PStringId, hash: u64, ex_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::STRLIT(contents_id, hash), ex_pos)
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

    pub fn new_unary_expr(op: &str, value: ExpressionNode, ex_pos: pos::Position) -> Self {
        match op {
            "-" => Self::new(ExpressionNodeKind::NEG(Box::new(value)), ex_pos),
            "&" => Self::new(ExpressionNodeKind::ADDRESS(Box::new(value)), ex_pos),
            "*" => Self::new(ExpressionNodeKind::DEREF(Box::new(value)), ex_pos),
            _ => unimplemented!()
        }
    }
    pub fn new_member(id: ExpressionNode, member: res::PStringId, ex_pos: pos::Position) -> Self {
        Self::new(ExpressionNodeKind::MEMBER(Box::new(id), member), ex_pos)
    }
    pub fn new_binary_expr(op: &str, lop: ExpressionNode, rop: ExpressionNode, ex_pos: pos::Position) -> Self {
        match op {
            "+" => Self::new(ExpressionNodeKind::ADD(Box::new(lop), Box::new(rop)), ex_pos),
            "-" => Self::new(ExpressionNodeKind::SUB(Box::new(lop), Box::new(rop)), ex_pos),
            "*" => Self::new(ExpressionNodeKind::MUL(Box::new(lop), Box::new(rop)), ex_pos),
            "/" => Self::new(ExpressionNodeKind::DIV(Box::new(lop), Box::new(rop)), ex_pos),
            "=" => Self::new(ExpressionNodeKind::ASSIGN(Box::new(lop), Box::new(rop)), ex_pos),
            _ => unimplemented!()
        }
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
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum ExpressionNodeKind {
    INTEGER(i64),
    UNSIGNEDINTEGER(u64),
    STRLIT(res::PStringId, u64),
    IDENT(IdentName),
    CALL(IdentName, Vec<ExpressionNode>),
    TRUE,
    FALSE,

    // unary
    ADDRESS(Box<ExpressionNode>),
    DEREF(Box<ExpressionNode>),
    NEG(Box<ExpressionNode>),
    MEMBER(Box<ExpressionNode>, res::PStringId),

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
            Self::UNSIGNEDINTEGER(v) => write!(f, "{}u", v),
            Self::STRLIT(contents, _hash) => write!(f, "\"{:?}\"", contents),
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
            Self::ADDRESS(v) => write!(f, "&{}", v),
            Self::DEREF(v) => write!(f, "*{}", v),
            Self::MEMBER(id, member) => write!(f, "{}.{:?}", id, member),

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

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct IdentName {
    pub name: res::PStringId,
    pub next: Option<Box<IdentName>>,
}

impl IdentName {
    fn new(base_id: res::PStringId, nxt: Option<Box<IdentName>>) -> Self {
        Self {
            name: base_id,
            next: nxt,
        }
    }

    pub fn new_terminated(base_id: res::PStringId) -> Self {
        Self::new(base_id, None)
    }

    pub fn append_next(&mut self, next: IdentName) {
        self.next = Some(Box::new(next));
    }

    pub fn correct_name(s: &IdentName) -> Vec<res::PStringId> {
        let mut corrected: Vec<res::PStringId> = Vec::new();
        corrected.push(s.name);

        let mut prev = &s.next;

        loop {
            if prev.is_none() {
                break;
            }

            corrected.push(prev.as_ref().unwrap().name);
            prev = &prev.as_ref().unwrap().next;
        }

        corrected
    }

    pub fn last_name(s: &IdentName) -> res::PStringId {
        let mut st = s.name;

        let mut prev = &s.next;
        loop {
            if prev.is_none() {
                break;
            }

            st = prev.as_ref().unwrap().name;
            prev = &prev.as_ref().unwrap().next;
        }

        st
    }
}

impl std::fmt::Display for IdentName {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", IdentName::correct_name(self))
    }
}
