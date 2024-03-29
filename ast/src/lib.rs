use types::{Span, ValType};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    Exponent,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UnaryOperator {
    Negate,
    Factorial,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Branch {
    pub cond_left: LocatedExpression,
    pub cond: types::CompareOperator,
    pub cond_right: LocatedExpression,
    pub val: LocatedExpression,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CallModifier {
    MapCall,
    NormalCall,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Function {
    Normal { name: String },
    Log { base: String },
}

// Expression is a component of a statement
#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Error,
    Num(String),
    Variable(String),
    RawLatex(ValType, String),
    FullyQualifiedVariable {
        path: Vec<String>,
        item: String,
    },
    BinaryExpr {
        left: Box<LocatedExpression>,
        // Should probably make an enum for this, but its not worth the work to encode
        //  it just to stringify it again later
        operator: BinaryOperator,
        right: Box<LocatedExpression>,
    },
    UnaryExpr {
        val: Box<LocatedExpression>,
        operator: UnaryOperator,
    },
    Map(Box<LocatedExpression>),
    Call {
        func: Function,
        args: Vec<LocatedExpression>,
    },
    List(Vec<LocatedExpression>),
    Range {
        first: Box<LocatedExpression>,
        second: Option<Box<LocatedExpression>>,
        end: Box<LocatedExpression>,
    },
    Piecewise {
        first: Box<Spanned<Branch>>,
        rest: Vec<Spanned<Branch>>,
        default: Box<LocatedExpression>,
    },
    Index {
        val: Box<LocatedExpression>,
        ind: Box<LocatedExpression>,
    },
}

pub type Spanned<T> = (Span, T);

pub type LocatedExpression = Spanned<Expression>;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub args: Vec<(Span, String, ValType)>,
    pub ret_annotation: Option<ValType>,
    pub inline: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImportMode {
    Import { name: String },
    Include,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Import {
    pub path: String,
    pub mode: ImportMode,
}

// A statement is a part of a program
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    VarDef {
        name: String,
        val: LocatedExpression,
        inline: bool,
    },
    FuncDef(FunctionDefinition, LocatedExpression),
    Expression(Expression),
    Import(Import),
}

pub type LocatedStatement = Spanned<Statement>;
pub type LStatements = Vec<LocatedStatement>;

// Syntax-dependent formatting functions are defined here rather than in the compiler

pub fn fmt_namespace(path: &Vec<String>) -> String {
    path.join(".")
}

pub fn func_name(func: Function) -> String {
    match func {
        Function::Normal { name } => name,
        Function::Log { base } => format!("log_{}", base),
    }
}
