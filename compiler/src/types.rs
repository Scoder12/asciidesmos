use ast::LStatements;
use std::{collections::HashMap, convert::TryFrom, fmt::Debug, rc::Rc};
use types::ValType;

use crate::{error::CompileError, stdlib::StdlibLoader};

// ValType that supports list mapping
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Typ {
    Num,
    List,
    MappedList,
}

impl std::fmt::Display for Typ {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Num => "number",
                Self::List => "list",
                Self::MappedList => "mapped list",
            }
        )
    }
}

impl From<ValType> for Typ {
    fn from(v: ValType) -> Self {
        match v {
            ValType::Number => Self::Num,
            ValType::List => Self::List,
        }
    }
}

impl TryFrom<Typ> for ValType {
    type Error = ();

    fn try_from(value: Typ) -> Result<Self, Self::Error> {
        match value {
            Typ::Num => Ok(ValType::Number),
            Typ::List => Ok(ValType::List),
            Typ::MappedList => Err(()),
        }
    }
}

impl Typ {
    pub fn is_num_weak(self) -> bool {
        match self {
            Self::Num => true,
            Self::List => false,
            Self::MappedList => true,
        }
    }

    pub fn is_list_weak(self) -> bool {
        match self {
            Self::Num => false,
            Self::List => true,
            Self::MappedList => true,
        }
    }

    pub fn is_num_strict(self) -> bool {
        self == Self::Num
    }

    pub fn eq_weak(self, rhs: Self) -> bool {
        match self {
            Self::Num => rhs.is_num_weak(),
            Self::List => rhs == Self::List,
            // todo: reject redundant cmp of mappedlist to mappedlist?
            Self::MappedList => rhs.is_num_weak(),
        }
    }
}

pub fn binop_exprs(
    left: (types::Span, Typ, TypInfo),
    right: (types::Span, Typ, TypInfo),
) -> (types::Span, Typ, TypInfo) {
    let (ls, lt, li) = left;
    let (rs, rt, ri) = right;
    if lt.is_list_weak() {
        return (ls, Typ::List, li);
    }
    if rt.is_list_weak() {
        return (rs, Typ::List, ri);
    }
    // only possibilities left:
    debug_assert_eq!(lt, Typ::Num);
    debug_assert_eq!(rt, Typ::Num);
    (
        ls.with_end_of(&rs).expect("Parsing same file"),
        lt,
        TypInfo::BinOp(ls, rs),
    )
}

pub fn combine_types(left: Typ, right: Typ) -> Typ {
    if left.is_list_weak() || right.is_list_weak() {
        return Typ::List;
    }
    // only possibilities left:
    debug_assert_eq!(left, Typ::Num);
    debug_assert_eq!(right, Typ::Num);
    Typ::Num
}

pub fn reduce_with_binop_exprs<I>(types: I) -> Option<(types::Span, Typ, TypInfo)>
where
    I: IntoIterator<Item = (types::Span, Typ, TypInfo)>,
{
    types
        .into_iter()
        .reduce(|left, right| binop_exprs(left, right))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Literal {
    Numeric,
    List,
    Range,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypInfo {
    Literal(Literal, types::Span),
    BinOp(types::Span, types::Span),
    Map(types::Span),
    Builtin(types::Span, ast::Function),
    RawLatex(types::Span),
    InlineFuncArg(types::Span),
    Call {
        call_span: types::Span,
        ret: Box<TypInfo>,
    },
    MappedCall {
        call_span: types::Span,
        func: ast::Function,
        mapped_arg: Box<TypInfo>,
    },
}

// heap version of core::runtime::Args
#[derive(Clone, Debug, PartialEq)]
pub enum FunctionArgs {
    Static(Vec<ValType>),
    Variadic,
}

// heap version of core::runtime::Function
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionSignature {
    pub args: FunctionArgs,
    pub ret: (ValType, TypInfo),
}

#[derive(Clone, Debug, PartialEq)]
pub struct InlineFunction {
    pub args: Vec<(String, ValType)>,
    pub ret: (Typ, TypInfo),
    pub body: latex::Latex,
}

pub trait Loader: LoaderClone + Debug {
    fn load(&self, path: &str) -> Option<LStatements>;

    fn parse_source(&self, source: &str) -> Option<LStatements>;
}

// https://stackoverflow.com/a/30353928/9196137
pub trait LoaderClone {
    fn box_clone(&self) -> Box<dyn Loader>;
}

impl<T> LoaderClone for T
where
    T: 'static + Loader + Clone,
{
    fn box_clone(&self) -> Box<dyn Loader> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn 'static + Loader> {
    fn clone(&self) -> Self {
        self.box_clone()
    }
}

#[derive(Clone, Debug)]
pub struct UnimplementedLoader;

impl Loader for UnimplementedLoader {
    fn load(&self, _path: &str) -> Option<LStatements> {
        unimplemented!()
    }

    fn parse_source(&self, _source: &str) -> Option<LStatements> {
        unimplemented!()
    }
}

impl Default for Box<dyn Loader> {
    fn default() -> Self {
        Box::new(UnimplementedLoader)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Context {
    pub variables: HashMap<String, (ValType, TypInfo)>,
    pub locals: HashMap<String, (ValType, TypInfo)>,
    pub defined_functions: HashMap<String, Rc<FunctionSignature>>,
    pub inline_vals: HashMap<String, (latex::Latex, Typ, TypInfo)>,
    pub inline_fns: HashMap<String, Rc<InlineFunction>>,
    // can't support submodules (yet)
    pub modules: HashMap<String, Context>,
    pub stdlib: StdlibLoader,
    pub loader: Box<dyn Loader>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn new_with_loader(loader: Box<dyn Loader>) -> Self {
        Self {
            loader,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResolvedFunction {
    Normal {
        func: Rc<FunctionSignature>,
        is_builtin: bool,
    },
    Inline(Rc<InlineFunction>),
}

pub type Cesult<T> = Result<T, CompileError>;
