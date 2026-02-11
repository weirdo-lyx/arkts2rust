#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stmt {
    Placeholder,
}
