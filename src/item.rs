#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Item {
    Literal(String),
    Expression(Expression),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Expression {
    pub operator: Option<Operator>,
    pub variable_list: Vec<Varspec>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Varspec {
    pub varname: String,
    pub modifier_level4: Option<ModifierLevel4>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operator {
    Reserved,
    Fragment,
    Label,
    PathSegment,
    PathParameter,
    FormQuery,
    FormContinuation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModifierLevel4 {
    Prefix(usize),
    Explode,
}
