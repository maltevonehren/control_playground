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
    TransferFunction(Vec<f64>, Vec<f64>),
    Tf2Ss(Box<Expression>),
    Load(Box<Expression>),
    Step(Box<Expression>),
}
