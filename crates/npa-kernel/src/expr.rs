use crate::level::Level;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Sort(Level),
    BVar(u32),
    Const {
        name: String,
        levels: Vec<Level>,
    },
    App(Box<Expr>, Box<Expr>),
    Lam {
        binder: String,
        ty: Box<Expr>,
        body: Box<Expr>,
    },
    Pi {
        binder: String,
        ty: Box<Expr>,
        body: Box<Expr>,
    },
    Let {
        binder: String,
        ty: Box<Expr>,
        value: Box<Expr>,
        body: Box<Expr>,
    },
}

impl Expr {
    pub fn sort(level: Level) -> Self {
        Self::Sort(level)
    }

    pub fn bvar(index: u32) -> Self {
        Self::BVar(index)
    }

    pub fn konst(name: impl Into<String>, levels: Vec<Level>) -> Self {
        Self::Const {
            name: name.into(),
            levels,
        }
    }

    pub fn app(fun: Self, arg: Self) -> Self {
        Self::App(Box::new(fun), Box::new(arg))
    }

    pub fn apps(fun: Self, args: impl IntoIterator<Item = Self>) -> Self {
        args.into_iter().fold(fun, Self::app)
    }

    pub fn lam(binder: impl Into<String>, ty: Self, body: Self) -> Self {
        Self::Lam {
            binder: binder.into(),
            ty: Box::new(ty),
            body: Box::new(body),
        }
    }

    pub fn pi(binder: impl Into<String>, ty: Self, body: Self) -> Self {
        Self::Pi {
            binder: binder.into(),
            ty: Box::new(ty),
            body: Box::new(body),
        }
    }

    pub fn let_in(binder: impl Into<String>, ty: Self, value: Self, body: Self) -> Self {
        Self::Let {
            binder: binder.into(),
            ty: Box::new(ty),
            value: Box::new(value),
            body: Box::new(body),
        }
    }
}

pub fn collect_apps(term: &Expr) -> (Expr, Vec<Expr>) {
    let mut args = Vec::new();
    let mut head = term.clone();
    while let Expr::App(fun, arg) = head {
        args.push(*arg);
        head = *fun;
    }
    args.reverse();
    (head, args)
}
