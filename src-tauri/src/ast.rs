use crate::lexer::Token;

// ─── Top Level ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Script {
    pub name: String,
    pub version: i64,
    pub items: Vec<ScriptItem>,
}

#[derive(Debug, Clone)]
pub enum ScriptItem {
    Var(VarDecl),
    Function(FunctionDef),
    OnStart(Vec<Stmt>),
}

// ─── Variables ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub name: String,
    pub init: Expr,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub name: String,
    pub value: Expr,
}

// ─── Functions ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
}

// ─── Statements ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Stmt {
    VarDecl(VarDecl),
    Assignment(Assignment),
    Return(Option<Expr>),
    If(IfStmt),
    Loop(Vec<Stmt>),
    While(WhileStmt),
    Break,
    Restart,
    Action(ActionStmt),
    ExprStmt(Expr),
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Condition,
    pub then_body: Vec<Stmt>,
    pub else_body: Option<Vec<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Vec<Stmt>,
}

// ─── Expressions ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    IntNumber(i64),
    String(String),
    Bool(bool),
    Color(String),
    Identifier(String),
    BinaryOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    UnaryNot(Box<Expr>),
    FuncCall {
        name: String,
        args: Vec<Expr>,
    },
    // Pixel color check used as an expression in conditions
    PixelColor {
        x: Box<Expr>,
        y: Box<Expr>,
    },
    RegionMatch {
        x: Box<Expr>,
        y: Box<Expr>,
        width: Box<Expr>,
        height: Box<Expr>,
        image_path: Box<Expr>,
        confidence: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Eq, Neq, Lt, Gt, Lte, Gte,
    And, Or,
}

// ─── Conditions ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Condition {
    PixelMatches {
        x: Box<Expr>,
        y: Box<Expr>,
        color: Box<Expr>,
        tolerance: Option<Box<Expr>>,
    },
    RegionMatches {
        x: Box<Expr>,
        y: Box<Expr>,
        width: Box<Expr>,
        height: Box<Expr>,
        image_path: Box<Expr>,
        confidence: Option<Box<Expr>>,
    },
    WaitUntil {
        condition: Box<Condition>,
        timeout: Option<Box<Expr>>,
        body: Vec<Stmt>,
        else_body: Option<Vec<Stmt>>,
    },
    Expression(Box<Expr>),
    Negated(Box<Condition>),
}

// ─── Actions ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ActionStmt {
    KeyTap { key: Expr },
    KeyHold { key: Expr },
    KeyRelease { key: Expr },
    KeyTypeText { text: Expr },
    MouseClick { button: Expr },
    MousePress { button: Expr },
    MouseRelease { button: Expr },
    MouseMove { x: Expr, y: Expr },
    Pause(PauseValue),
    Call { name: String, args: Vec<Expr> },
}

#[derive(Debug, Clone)]
pub enum PauseValue {
    Fixed(Expr),
    Range { min: Box<Expr>, max: Box<Expr> },
    Human,
}

// ─── Parse Context (for error reporting) ──────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Span {
    pub line: usize,
    pub col: usize,
}

impl Span {
    pub fn from_token(tok: &Token) -> Self {
        Self { line: tok.line, col: tok.col }
    }
}