use crate::encoding::{push_allow_unreserved, push_allow_unreserved_reserved, PushAllow};
use crate::item::{Expression, Item, ModifierLevel4, Operator, Varspec};
use crate::{Value, Variables};

pub fn expand_items<V>(items: &[Item], variables: &V) -> String
where
    V: Variables,
{
    let mut dst = String::new();
    for item in items {
        match item {
            Item::Literal(literal) => expand_literal(&mut dst, literal),
            Item::Expression(expression) => expand_expression(variables, &mut dst, expression),
        }
    }
    dst
}

fn expand_literal(dst: &mut String, literal: &str) {
    dst.push_str(literal);
}

fn expand_expression<V>(variables: &V, dst: &mut String, expression: &Expression)
where
    V: Variables,
{
    let operator_table = get_operator_table(expression.operator);
    let mut push_sep = make_push_sep(operator_table.first, operator_table.sep);
    for varspec in &expression.variable_list {
        if let Some(value) = variables.get(&varspec.varname) {
            if let Some(ModifierLevel4::Explode) = varspec.modifier_level4 {
                explode_varspec(dst, &operator_table, &mut push_sep, varspec, value);
            } else {
                expand_varspec(dst, &operator_table, &mut push_sep, varspec, value);
            }
        }
    }
}

struct OperatorTable {
    first: &'static str,
    sep: &'static str,
    named: bool,
    ifemp: &'static str,
    allow: PushAllow,
}

fn get_operator_table(operator: Option<Operator>) -> OperatorTable {
    match operator {
        None => OperatorTable {
            first: "",
            sep: ",",
            named: false,
            ifemp: "",
            allow: push_allow_unreserved,
        },
        Some(Operator::Reserved) => OperatorTable {
            first: "",
            sep: ",",
            named: false,
            ifemp: "",
            allow: push_allow_unreserved_reserved,
        },
        Some(Operator::Fragment) => OperatorTable {
            first: "#",
            sep: ",",
            named: false,
            ifemp: "",
            allow: push_allow_unreserved_reserved,
        },
        Some(Operator::Label) => OperatorTable {
            first: ".",
            sep: ".",
            named: false,
            ifemp: "",
            allow: push_allow_unreserved,
        },
        Some(Operator::PathSegment) => OperatorTable {
            first: "/",
            sep: "/",
            named: false,
            ifemp: "",
            allow: push_allow_unreserved,
        },
        Some(Operator::PathParameter) => OperatorTable {
            first: ";",
            sep: ";",
            named: true,
            ifemp: "",
            allow: push_allow_unreserved,
        },
        Some(Operator::FormQuery) => OperatorTable {
            first: "?",
            sep: "&",
            named: true,
            ifemp: "=",
            allow: push_allow_unreserved,
        },
        Some(Operator::FormContinuation) => OperatorTable {
            first: "&",
            sep: "&",
            named: true,
            ifemp: "=",
            allow: push_allow_unreserved,
        },
    }
}

fn make_push_sep(first: &'static str, sep: &'static str) -> impl FnMut(&mut String) {
    let mut s = first;
    move |dst: &mut String| {
        dst.push_str(s);
        s = sep;
    }
}

fn explode_varspec<F>(
    dst: &mut String,
    operator_table: &OperatorTable,
    push_sep: &mut F,
    varspec: &Varspec,
    value: &Value,
) where
    F: FnMut(&mut String),
{
    match value {
        Value::AssociativeArray(value) if !value.is_empty() => {
            push_sep(dst);
            explode_varspec_assoc(dst, operator_table, value);
        }
        Value::List(value) if !value.is_empty() => {
            push_sep(dst);
            explode_varspec_list(dst, operator_table, varspec, value);
        }
        _ => {}
    }
}

fn explode_varspec_assoc(
    dst: &mut String,
    operator_table: &OperatorTable,
    value: &[(String, String)],
) {
    let mut push_sep = make_push_sep("", operator_table.sep);
    if !operator_table.named {
        expand_assoc(dst, &operator_table.allow, &mut push_sep, "=", value);
    } else {
        for (k, v) in value {
            push_sep(dst);
            push_allow_unreserved(dst, k);
            if v.is_empty() {
                dst.push_str(operator_table.ifemp);
            } else {
                dst.push('=');
                (operator_table.allow)(dst, v);
            }
        }
    }
}

fn explode_varspec_list(
    dst: &mut String,
    operator_table: &OperatorTable,
    varspec: &Varspec,
    value: &[String],
) {
    let mut push_sep = make_push_sep("", operator_table.sep);
    if !operator_table.named {
        expand_list(dst, &operator_table.allow, &mut push_sep, value);
    } else {
        for v in value {
            push_sep(dst);
            push_allow_unreserved(dst, &varspec.varname);
            if v.is_empty() {
                dst.push_str(operator_table.ifemp);
            } else {
                dst.push('=');
                (operator_table.allow)(dst, v);
            }
        }
    }
}

fn expand_varspec<F>(
    dst: &mut String,
    operator_table: &OperatorTable,
    push_sep: &mut F,
    varspec: &Varspec,
    value: &Value,
) where
    F: FnMut(&mut String),
{
    match value {
        Value::AssociativeArray(value) if !value.is_empty() => {
            push_sep(dst);
            expand_varspec_assoc(dst, operator_table, varspec, value);
        }
        Value::List(value) if !value.is_empty() => {
            push_sep(dst);
            expand_varspec_list(dst, operator_table, varspec, value);
        }
        Value::String(value) => {
            push_sep(dst);
            expand_varspec_string(dst, operator_table, varspec, value);
        }
        _ => {}
    }
}

fn expand_varspec_assoc(
    dst: &mut String,
    operator_table: &OperatorTable,
    varspec: &Varspec,
    value: &[(String, String)],
) {
    push_name(dst, operator_table, varspec, false);
    let mut push_sep = make_push_sep("", ",");
    expand_assoc(dst, &operator_table.allow, &mut push_sep, ",", value);
}

fn expand_varspec_list(
    dst: &mut String,
    operator_table: &OperatorTable,
    varspec: &Varspec,
    value: &[String],
) {
    push_name(dst, operator_table, varspec, false);
    let mut push_sep = make_push_sep("", ",");
    expand_list(dst, &operator_table.allow, &mut push_sep, value);
}

fn expand_varspec_string(
    dst: &mut String,
    operator_table: &OperatorTable,
    varspec: &Varspec,
    mut value: &str,
) {
    let empty = value.is_empty();
    push_name(dst, operator_table, varspec, empty);
    if !empty {
        if let Some(ModifierLevel4::Prefix(size)) = varspec.modifier_level4 {
            let i = value
                .char_indices()
                .map(|(i, _)| i)
                .take(size)
                .last()
                .unwrap_or(0);
            value = &value[..=i];
        }
        (operator_table.allow)(dst, value);
    }
}

fn push_name(dst: &mut String, operator_table: &OperatorTable, varspec: &Varspec, empty: bool) {
    if operator_table.named {
        push_allow_unreserved(dst, &varspec.varname);
        if empty {
            dst.push_str(operator_table.ifemp);
        } else {
            dst.push('=')
        }
    }
}

fn expand_assoc<F>(
    dst: &mut String,
    push_allow: &PushAllow,
    push_sep: &mut F,
    kv_sep: &str,
    value: &[(String, String)],
) where
    F: FnMut(&mut String),
{
    for (k, v) in value {
        push_sep(dst);
        push_allow(dst, k);
        dst.push_str(kv_sep);
        push_allow(dst, v);
    }
}

fn expand_list<F>(dst: &mut String, push_allow: &PushAllow, push_sep: &mut F, value: &[String])
where
    F: FnMut(&mut String),
{
    for v in value {
        push_sep(dst);
        push_allow(dst, v);
    }
}
