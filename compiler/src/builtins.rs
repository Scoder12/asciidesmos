use phf::{phf_map, Map};
use types::{
    Args, Function,
    ValType::{List, Number as Num},
};

macro_rules! f {
    ($args:expr, $ret:expr) => {
        Function {
            args: Args::Static($args),
            ret: $ret,
        }
    };
}

macro_rules! n {
    () => {
        f!(&[Num], Num)
    };
}

macro_rules! nn {
    () => {
        f!(&[Num, Num], Num)
    };
}

macro_rules! l {
    () => {
        f!(&[List], Num)
    };
}

macro_rules! ll {
    () => {
        f!(&[List, List], Num)
    };
}

// Map of desmos builtin functions.
// Source: https://support.desmos.com/hc/en-us/articles/212235786-Supported-Functions
pub static BUILTIN_FUNCTIONS: Map<&'static str, Function> = phf_map! {
    // Trigonometry
    "sin" => n!(),
    "cos" => n!(),
    "tan" => n!(),
    "csc" => n!(),
    "sec" => n!(),
    "cot" => n!(),

    "arcsin" => n!(),
    "arccos" => n!(),
    "arctan" => n!(),
    "arccsc" => n!(),
    "arcsec" => n!(),
    "arccot" => n!(),

    "sinh" => n!(),
    "cosh" => n!(),
    "tanh" => n!(),
    "csch" => n!(),
    "sech" => n!(),
    "coth" => n!(),

    // Statistics
    "total" => l!(),
    "min" => l!(),
    "max" => l!(),
    "length" => l!(),
    "mean" => l!(),
    "median" => l!(),
    "stdev" => l!(),
    "stdevp" => l!(),
    "mad" => l!(),
    "var" => l!(),
    "cov" => l!(),

    "corr" => ll!(),

    "quantile" => f!(&[List, Num], Num),

    "nCr" => nn!(),
    "nPr" => nn!(),

    // Miscellaneous
    "join" => ll!(),

    "sort" => l!(),
    "shuffle" => l!(),

    "lcm" => Function {
        args: Args::Variadic,
        ret: Num,
    },
    "gcd" => Function {
        args: Args::Variadic,
        ret: Num,
    },

    "mod" => nn!(),

    "floor" => n!(),
    "abs" => n!(),
    "sign" => n!(), // returns 1, -1, or 0 based on sign
    "exp" => n!(), // e^x
    "ln" => n!(),
    "log" => n!(),
    // log_{base} is supported through a special case in the parser

    // supported through a special case in IR output
    "sqrt" => n!(),
    "nthroot" => nn!(),

    // TODO: Support integral
    // TODO: Support sum
    // TODO: Support prod
    // theses are going to be pretty hard as they require
    //  adding variables to scope based on presence of d$var

    // TODO: Support for optional arguments.
    // Round takes either one or two arguments
    "round" => n!(),

};
