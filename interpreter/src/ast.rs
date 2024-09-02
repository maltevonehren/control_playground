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
    TransferFunction(Vec<f64>, Vec<f64>),
    TF2SS(String),
}
