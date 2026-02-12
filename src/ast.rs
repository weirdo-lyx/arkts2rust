/// 整个程序（Program）的 AST 节点。
///
/// AST（抽象语法树）是“语法结构的树形表示”，它比 Token 流更接近我们对代码结构的理解：
/// - Token 流：`let`、`x`、`=`、`1`、`;`（一串积木）
/// - AST：一条“变量声明语句”，名字是 x，初始值是数字 1（有结构）
///
/// 目前（Step2~Step5）只支持最小语句集，所以 Program 里只是一组 `Stmt`。
///
/// 说明：为了保持最小实现，这里的 AST 节点暂不保存 Span。
/// 错误定位主要由 Parser 在报错时提供（使用当前 Token 的 Span）。
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

/// 语句（Statement）枚举。
///
/// 本项目的“语句”就是一条可以独立执行的代码，且在 Step2 的语法里每条语句必须以 `;` 结尾。
/// 由于 `;` 只是语法细节，不影响语义结构，所以 AST 里不显式保存分号。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stmt {
    /// 变量声明：`let x = 1;` 或 `const y = "abc";`
    VarDecl(VarDecl),
    /// 赋值语句：`x = expr;`
    ///
    /// 注意：这里把赋值当作“语句”而不是“表达式”，是为了保持 Step4 的范围最小：
    /// - 不支持像 `a = b = 1;` 这种链式赋值表达式
    /// - 只支持最常见的 `Ident = Expr ;`
    Assign(AssignStmt),
    /// 表达式语句：`console.log(123);`
    ExprStmt(Expr),
    /// 代码块：`{ stmt* }`
    ///
    /// 代码块本身是一条语句，但内部可以再嵌套任意条语句（包括 if/while/block）。
    Block(BlockStmt),
    /// if/else 语句：`if (cond) stmt else stmt`
    If(IfStmt),
    /// while 语句：`while (cond) stmt`
    While(WhileStmt),
    /// return 语句：`return expr?;`
    ///
    /// 注意：由于我们把所有代码都生成到 `fn main() { ... }` 里，
    /// Rust 的 main 返回类型是 `()`，因此 `return <expr>;` 的“返回值”在 Rust 中没有意义。
    /// CodeGen 会把它当作“提前结束”处理：先计算 expr（若存在），再 `return;`。
    Return(ReturnStmt),
}

/// 变量声明结构体（let/const）。
///
/// Step2 限制：初始化表达式只允许是字面量（Literal）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VarDecl {
    /// 是否为常量（const 为 true，let 为 false）
    pub is_const: bool,
    /// 变量名
    pub name: String,
    /// 初始值（目前只支持字面量）
    pub init: Literal,
}

/// 赋值语句结构体：`name = value;`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssignStmt {
    pub name: String,
    pub value: Expr,
}

/// 代码块结构体：`{ stmt* }`
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockStmt {
    pub stmts: Vec<Stmt>,
}

/// if/else 结构体。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfStmt {
    pub cond: Expr,
    pub then_branch: Box<Stmt>,
    pub else_branch: Box<Stmt>,
}

/// while 结构体。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WhileStmt {
    pub cond: Expr,
    pub body: Box<Stmt>,
}

/// return 结构体：可选返回值。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
}

/// 表达式（Expression）枚举。
///
/// Step2/Step3 的最小表达式集：
/// - 字面量：number/string/boolean
/// - 函数调用：仅支持 console.log(literal)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    /// 字面量表达式：123, "abc", true
    Literal(Literal),
    /// 标识符引用：`x`
    Ident(String),
    /// 一元运算：`!x`、`-x`
    Unary(UnaryExpr),
    /// 二元运算：`a + b`、`a && b` 等
    Binary(BinaryExpr),
    /// 括号表达式：`(a + b)`
    ///
    /// 说明：如果不把括号保存进 AST，CodeGen 很容易丢失用户写的括号，导致语义变化。
    Group(Box<Expr>),
    /// 函数调用表达式：console.log(...)
    Call(CallExpr),
}

/// 一元表达式结构体。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
}

/// 一元运算符枚举。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Neg,
}

/// 二元表达式结构体。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

/// 二元运算符枚举。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    EqEq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    AndAnd,
    OrOr,
}

/// 函数调用表达式结构体。
///
/// Step2 约束：只支持一个参数，并且参数必须是字面量。
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
    Ident(String),
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
