#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stmt {
    VarDecl(VarDecl),
    ExprStmt(Expr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarDecl {
    pub is_const: bool,
    pub name: String,
    pub init: Literal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Literal(Literal),
    Call(CallExpr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallExpr {
    pub callee: Callee,
    pub args: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Callee {
    ConsoleLog,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal {
    Number(i32),
    String(String),
    Bool(bool),
}
