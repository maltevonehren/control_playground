#[derive(Clone, Debug, Default, PartialEq)]
pub struct Program {
    pub(crate) statements: Vec<Statement>,
}

#[derive(Clone, PartialEq, Debug)]
pub(crate) enum Statement {
    ExpressionStatement(Expression),
    Assign(String, Expression),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expression {
    Identifier(String),
    StringLiteral(String),
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
    pub input_name: String,
    pub item_name: String,
    pub output_name: String,
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
