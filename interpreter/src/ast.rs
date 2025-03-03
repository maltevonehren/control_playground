use std::rc::Rc;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Program {
    pub(crate) statements: Vec<Statement>,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Statement {
    ExpressionStatement(Expression),
    Assign(Rc<str>, Expression),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expression {
    Identifier(Rc<str>),
    StringLiteral(Rc<str>),
    FloatLiteral(f64),
    VectorLiteral(Vec<Expression>),
    UnOp(UnOp, Box<Expression>),
    BinOp(BinOp, Box<Expression>, Box<Expression>),
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
    System(Vec<SystemItem>),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SystemItem {
    pub output_name: Rc<str>,
    pub rhs: SystemItemRhs,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SystemItemRhs {
    Difference {
        input1_name: Rc<str>,
        input2_name: Rc<str>,
    },
    System {
        system_name: Rc<str>,
        input_name: Rc<str>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum UnOp {
    Neg,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}
