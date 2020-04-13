use crate::common::position as pos;
use crate::compiler::resource as res;

#[derive(Clone)]
pub struct StatementNode {
    pub kind: StatementNodeKind,
    position: pos::Position,
}

impl StatementNode {
    fn new(stmt_kind: StatementNodeKind, stmt_pos: pos::Position) -> Self {
        Self {
            kind: stmt_kind,
            position: stmt_pos,
        }
    }

    pub fn new_return(expr: res::ExpressionNode, st_pos: pos::Position) -> Self {
        Self::new(StatementNodeKind::RETURN(expr), st_pos)
    }

    pub fn new_ifret(expr: res::ExpressionNode, st_pos: pos::Position) -> Self {
        Self::new(StatementNodeKind::IFRET(expr), st_pos)
    }

    pub fn new_expr(expr: res::ExpressionNode, st_pos: pos::Position) -> Self {
        Self::new(StatementNodeKind::EXPR(expr), st_pos)
    }

    pub fn new_vardecl(st_pos: pos::Position) -> Self {
        Self::new(StatementNodeKind::VARDECL, st_pos)
    }

    pub fn new_countup(
        ident: res::ExpressionNode,
        start_expr: res::ExpressionNode,
        end_expr: res::ExpressionNode,
        body: Vec<StatementNode>,
        st_pos: pos::Position,
    ) -> Self {
        Self::new(
            StatementNodeKind::COUNTUP(ident, start_expr, end_expr, body),
            st_pos,
        )
    }
}

impl std::fmt::Display for StatementNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} {}", self.position, self.kind)
    }
}

#[derive(Clone)]
pub enum StatementNodeKind {
    RETURN(res::ExpressionNode),
    IFRET(res::ExpressionNode),
    EXPR(res::ExpressionNode),
    VARDECL,
    COUNTUP(
        res::ExpressionNode,
        res::ExpressionNode,
        res::ExpressionNode,
        Vec<StatementNode>,
    ),
}

impl std::fmt::Display for StatementNodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::RETURN(inner) => write!(f, "return {};", inner),
            Self::IFRET(inner) => write!(f, "ifret {};", inner),
            Self::EXPR(inner) => write!(f, "expr {};", inner),
            Self::VARDECL => write!(f, "(vardecl);"),
            Self::COUNTUP(ident, start, end, body) => {
                write!(f, "countup {} <= {} <= {} {{ \n", start, ident, end)?;

                for st in body.iter() {
                    write!(f, "\t\t{}\n", st)?;
                }

                write!(f, "}};")
            }
        }
    }
}
