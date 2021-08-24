use std::str::Chars;

use crate::encoding::{is_alpha, is_digit, is_hexdig};
use crate::item::{Expression, Item, ModifierLevel4, Operator, Varspec};

pub fn parse_template(mut template: &str) -> Vec<Item> {
    let mut items = Vec::new();
    while !template.is_empty() {
        match template.split_once('{') {
            None => {
                let item = parse_literal(template);
                items.push(item);
                break;
            }
            Some((literal, remainder)) => {
                if !literal.is_empty() {
                    let item = parse_literal(literal);
                    items.push(item);
                }
                match remainder.split_once('}') {
                    None => {
                        let mut s = String::new();
                        s.push('{');
                        s.push_str(remainder);
                        let item = Item::Literal(s);
                        items.push(item);
                        break;
                    }
                    Some((expression, remainder)) => {
                        let item = parse_expression(expression).unwrap_or_else(|_| {
                            let mut s = String::new();
                            s.push('{');
                            s.push_str(expression);
                            s.push('}');
                            Item::Literal(s)
                        });
                        items.push(item);
                        template = remainder;
                    }
                }
            }
        }
    }
    items
}

fn parse_literal(s: &str) -> Item {
    Item::Literal(s.to_string())
}

fn parse_expression(mut s: &str) -> Result<Item, ()> {
    let mut chars = s.chars();
    match chars.next() {
        None => Err(()),
        Some(operator) => {
            let operator = match operator {
                '+' => Some(Operator::Reserved),
                '#' => Some(Operator::Fragment),
                '.' => Some(Operator::Label),
                '/' => Some(Operator::PathSegment),
                ';' => Some(Operator::PathParameter),
                '?' => Some(Operator::FormQuery),
                '&' => Some(Operator::FormContinuation),
                _ => None,
            };
            if operator.is_some() {
                s = chars.as_str();
            }
            let variable_list = parse_variable_list(s)?;
            let expression = Expression {
                operator,
                variable_list,
            };
            let item = Item::Expression(expression);
            Ok(item)
        }
    }
}

fn parse_variable_list(s: &str) -> Result<Vec<Varspec>, ()> {
    s.split(',').map(parse_varspec).collect()
}

fn parse_varspec(s: &str) -> Result<Varspec, ()> {
    if s.is_empty() {
        Err(())
    } else {
        let asterisk = s.find('*');
        let colon = s.find(':');
        match (asterisk, colon) {
            (None, None) => {
                let varname = parse_varname(s)?;
                let varspec = Varspec {
                    varname,
                    modifier_level4: None,
                };
                Ok(varspec)
            }
            (None, Some(colon)) => {
                let varname = &s[..colon];
                let size = &s[colon + 1..];
                let varname = parse_varname(varname)?;
                let mut chars = size.chars();
                match chars.next() {
                    None | Some('0') => Err(()),
                    _ => {
                        let size = size.parse().map_err(|_| ())?;
                        if size >= 10000 {
                            Err(())
                        } else {
                            let varspec = Varspec {
                                varname,
                                modifier_level4: Some(ModifierLevel4::Prefix(size)),
                            };
                            Ok(varspec)
                        }
                    }
                }
            }
            (Some(asterisk), None) => {
                if asterisk != s.len() - 1 {
                    Err(())
                } else {
                    let varname = parse_varname(&s[..asterisk])?;
                    let varspec = Varspec {
                        varname,
                        modifier_level4: Some(ModifierLevel4::Explode),
                    };
                    Ok(varspec)
                }
            }
            _ => Err(()),
        }
    }
}

fn parse_varname(s: &str) -> Result<String, ()> {
    let mut chars = s.chars();
    match chars.next() {
        Some('%') => {
            require_pct_encoded(&mut chars)?;
        }
        Some(c) if is_varchar(c) => {}
        _ => {
            return Err(());
        }
    }
    loop {
        match chars.next() {
            None => {
                break;
            }
            Some('%') => {
                require_pct_encoded(&mut chars)?;
            }
            Some(c) if is_varchar(c) || '.' == c => {}
            _ => {
                return Err(());
            }
        }
    }
    Ok(s.to_string())
}

fn require_pct_encoded(chars: &mut Chars) -> Result<(), ()> {
    if is_pct_encoded(chars) {
        Ok(())
    } else {
        Err(())
    }
}

fn is_pct_encoded(chars: &mut Chars) -> bool {
    let x = chars.next();
    let y = chars.next();
    matches!((x, y), (Some(x), Some(y)) if is_hexdig(x) && is_hexdig(y))
}

fn is_varchar(c: char) -> bool {
    is_alpha(c) || is_digit(c) || '_' == c
}
