#[derive(Clone, PartialEq, Default, Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Statement {
    ExpressionStatement(Expression),
    Assign(String, Expression),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Expression {
    Identifier(String),
    StringLiteral(String),
    VectorLiteral(Vec<f64>),
    FunctionCall {
        function: Box<Expression>,
        arguments: Vec<Expression>,
    },
}
