/// 整个程序（Program）的 AST 节点。
/// 目前（Step2）只包含一系列语句。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

/// 语句（Statement）枚举。
/// 支持变量声明（let/const）和表达式语句（console.log）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stmt {
    /// 变量声明：`let x = 1;` 或 `const y = "abc";`
    VarDecl(VarDecl),
    /// 表达式语句：`console.log(123);`
    ExprStmt(Expr),
}

/// 变量声明结构体。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarDecl {
    /// 是否为常量（const 为 true，let 为 false）
    pub is_const: bool,
    /// 变量名
    pub name: String,
    /// 初始值（目前只支持字面量）
    pub init: Literal,
}

/// 表达式（Expression）枚举。
/// 目前支持字面量和函数调用。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    /// 字面量表达式：123, "abc", true
    Literal(Literal),
    /// 函数调用表达式：console.log(...)
    Call(CallExpr),
}

/// 函数调用表达式结构体。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CallExpr {
    /// 被调用的函数（目前只能是 console.log）
    pub callee: Callee,
    /// 参数列表（目前只支持一个参数）
    pub args: Vec<Expr>,
}

/// 被调用者枚举。
/// Step2 仅支持 `console.log`。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Callee {
    ConsoleLog,
}

/// 字面量（Literal）枚举。
/// 对应 ArkTS 的基础类型值。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Literal {
    /// 数字字面量（i32）
    Number(i32),
    /// 字符串字面量
    String(String),
    /// 布尔字面量
    Bool(bool),
}
