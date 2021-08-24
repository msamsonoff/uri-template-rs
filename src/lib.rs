mod encoding;
mod expand;
mod item;
mod parse;

use std::borrow::Borrow;
use std::collections::HashMap;

use crate::expand::expand_items;
use crate::item::Item;
use crate::parse::parse_template;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UriTemplate(Vec<Item>);

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    AssociativeArray(Vec<(String, String)>),
    List(Vec<String>),
    String(String),
}

pub trait Variables<'a, B>
where
    B: Borrow<Value>,
{
    fn get(&'a self, k: &'a str) -> Option<B>;
}

#[derive(Debug)]
pub struct Expander<'a> {
    uri_template: &'a UriTemplate,
    variables: HashMap<String, Value>,
}

impl UriTemplate {
    pub fn parse<S>(template: S) -> Self
    where
        S: AsRef<str>,
    {
        let template = template.as_ref();
        let items = parse_template(template);
        UriTemplate(items)
    }

    pub fn expand<'a, V, B>(&'a self, variables: &'a V) -> String
    where
        V: Variables<'a, B>,
        B: Borrow<Value>,
    {
        expand_items(&self.0, variables)
    }

    pub fn expander(&self) -> Expander {
        Expander {
            uri_template: self,
            variables: HashMap::new(),
        }
    }
}

impl Expander<'_> {
    pub fn expand(&self) -> String {
        self.uri_template.expand(&self.variables)
    }

    pub fn set_assoc<K1, V1, K2, V2>(&mut self, k1: K1, iter: V1) -> &mut Self
    where
        K1: Into<String>,
        V1: IntoIterator<Item = (K2, V2)>,
        K2: Into<String>,
        V2: Into<String>,
    {
        let k1 = k1.into();
        let v1 = Value::from_assoc(iter);
        self.variables.insert(k1, v1);
        self
    }

    pub fn set_list<K1, V1, V2>(&mut self, k: K1, iter: V1) -> &mut Self
    where
        K1: Into<String>,
        V1: IntoIterator<Item = V2>,
        V2: Into<String>,
    {
        let k = k.into();
        let v = Value::from_list(iter);
        self.variables.insert(k, v);
        self
    }

    pub fn set_string<K, V>(&mut self, k: K, v: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        let k = k.into();
        let v = Value::from_string(v);
        self.variables.insert(k, v);
        self
    }
}

impl Value {
    pub fn from_assoc<I, K, V>(iter: I) -> Value
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Value::AssociativeArray(
            iter.into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        )
    }

    pub fn from_list<I, V>(iter: I) -> Value
    where
        I: IntoIterator<Item = V>,
        V: Into<String>,
    {
        Value::List(iter.into_iter().map(Into::into).collect())
    }

    pub fn from_string<S>(s: S) -> Value
    where
        S: Into<String>,
    {
        Value::String(s.into())
    }
}

impl<'a> Variables<'a, &'a Value> for Vec<(String, Value)> {
    fn get(&'a self, k: &str) -> Option<&Value> {
        self.iter().find(|(k1, _)| k == k1).map(|(_, v1)| v1)
    }
}

impl<'a> Variables<'a, &'a Value> for HashMap<String, Value> {
    fn get(&self, k: &str) -> Option<&Value> {
        self.get(k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let left = UriTemplate::parse("").expander().expand();
        assert_eq!(left, "");
    }

    #[test]
    fn test_literal_expression_literal() {
        let left = UriTemplate::parse("x{y}z")
            .expander()
            .set_string("y", "Y")
            .expand();
        assert_eq!(left, "xYz");
    }

    #[test]
    fn test_expression_literal_expression() {
        let left = UriTemplate::parse("{x}y{z}")
            .expander()
            .set_string("x", "X")
            .set_string("z", "Z")
            .expand();
        assert_eq!(left, "XyZ");
    }

    #[test]
    fn test_literal() {
        let left = UriTemplate::parse("x").expander().expand();
        assert_eq!(left, "x");
    }

    #[test]
    fn test_expression() {
        let left = UriTemplate::parse("{x}")
            .expander()
            .set_string("x", "X")
            .expand();
        assert_eq!(left, "X");
    }

    #[test]
    fn test_expression_multiple_variables() {
        let left = UriTemplate::parse("{x,y}")
            .expander()
            .set_string("x", "X")
            .set_string("y", "Y")
            .expand();
        assert_eq!(left, "X,Y");
    }

    #[test]
    fn test_multiple_expressions_multiple_variables() {
        let left = UriTemplate::parse("{x}{y,z}")
            .expander()
            .set_string("x", "X")
            .set_string("y", "Y")
            .set_string("z", "Z")
            .expand();
        assert_eq!(left, "XY,Z")
    }

    #[test]
    fn test_varname_dots() {
        let left = UriTemplate::parse("{x.y.z}")
            .expander()
            .set_string("x.y.z", "X.Y.Z")
            .expand();
        assert_eq!(left, "X.Y.Z");
    }

    #[test]
    fn test_varname_pct_encoded() {
        let left = UriTemplate::parse("{%20%21}")
            .expander()
            .set_string("%20%21", "SPACE!")
            .expand();
        assert_eq!(left, "SPACE%21");
    }

    #[test]
    fn test_prefix() {
        let left = UriTemplate::parse("{x:2}")
            .expander()
            .set_string("x", "ABCD")
            .expand();
        assert_eq!(left, "AB");
    }

    #[test]
    fn test_empty_expression() {
        let left = UriTemplate::parse("{}").expander().expand();
        assert_eq!(left, "{}");
    }

    #[test]
    fn test_invalid_expression() {
        let left = UriTemplate::parse("{x").expander().expand();
        assert_eq!(left, "{x");
    }

    #[test]
    fn test_invalid_operator() {
        let left = UriTemplate::parse("{!x}").expander().expand();
        assert_eq!(left, "{!x}");
    }

    #[test]
    fn test_invalid_varspec() {
        let left = UriTemplate::parse("{x,,y}").expander().expand();
        assert_eq!(left, "{x,,y}");
    }

    #[test]
    fn test_invalid_varname() {
        let left = UriTemplate::parse("{?~}").expander().expand();
        assert_eq!(left, "{?~}");

        let left = UriTemplate::parse("{?.}").expander().expand();
        assert_eq!(left, "{?.}");

        let left = UriTemplate::parse("{?x~}").expander().expand();
        assert_eq!(left, "{?x~}");
    }

    #[test]
    fn test_invalid_prefix() {
        let left = UriTemplate::parse("{x:1y}").expander().expand();
        assert_eq!(left, "{x:1y}");

        let left = UriTemplate::parse("{x:-1}").expander().expand();
        assert_eq!(left, "{x:-1}");

        let left = UriTemplate::parse("{x:10000}").expander().expand();
        assert_eq!(left, "{x:10000}");
    }

    #[test]
    fn test_invalid_explode() {
        let left = UriTemplate::parse("{x*y}").expander().expand();
        assert_eq!(left, "{x*y}");
    }

    #[test]
    fn test_invalid_prefix_explode() {
        let left = UriTemplate::parse("{x:1*}").expander().expand();
        assert_eq!(left, "{x:1*}");
    }

    #[test]
    fn test_varname_invalid_pct_encoded() {
        let left = UriTemplate::parse("{%0}").expander().expand();
        assert_eq!(left, "{%0}");
    }

    #[test]
    fn test_expand_no_operator() {
        let left = UriTemplate::parse("{x}")
            .expander()
            .set_string("x", "A :B")
            .expand();
        assert_eq!(left, "A%20%3AB");
    }

    #[test]
    fn test_expand_reserved() {
        let left = UriTemplate::parse("{+x}")
            .expander()
            .set_string("x", "A :B")
            .expand();
        assert_eq!(left, "A%20:B");
    }

    #[test]
    fn test_expand_fragment() {
        let left = UriTemplate::parse("{#x}")
            .expander()
            .set_string("x", "A :B")
            .expand();
        assert_eq!(left, "#A%20:B");
    }

    #[test]
    fn test_expand_label() {
        let left = UriTemplate::parse("{.x}")
            .expander()
            .set_string("x", "A :B")
            .expand();
        assert_eq!(left, ".A%20%3AB");
    }

    #[test]
    fn test_expand_path_segment() {
        let left = UriTemplate::parse("{/x}")
            .expander()
            .set_string("x", "A :B")
            .expand();
        assert_eq!(left, "/A%20%3AB");
    }

    #[test]
    fn test_expand_path_operator() {
        let left = UriTemplate::parse("{;x}")
            .expander()
            .set_string("x", "A :B")
            .expand();
        assert_eq!(left, ";x=A%20%3AB");
    }

    #[test]
    fn test_expand_form_query() {
        let left = UriTemplate::parse("{?x}")
            .expander()
            .set_string("x", "A :B")
            .expand();
        assert_eq!(left, "?x=A%20%3AB");
    }

    #[test]
    fn test_expand_form_continuation() {
        let left = UriTemplate::parse("{&x}")
            .expander()
            .set_string("x", "A :B")
            .expand();
        assert_eq!(left, "&x=A%20%3AB");
    }

    #[test]
    fn test_expand_unnamed_operator() {
        let uri_template = UriTemplate::parse("x{+y}z");

        let left = uri_template.expander().set_string("y", "Y").expand();
        assert_eq!(left, "xYz");

        let left = uri_template.expander().set_string("y", "").expand();
        assert_eq!(left, "xz");

        let left = uri_template.expander().expand();
        assert_eq!(left, "xz");

        let left = uri_template
            .expander()
            .set_list("y", ["A", "", "B"])
            .expand();
        assert_eq!(left, "xA,,Bz");

        let left = uri_template
            .expander()
            .set_list("y", [] as [&str; 0])
            .expand();
        assert_eq!(left, "xz");

        let left = uri_template
            .expander()
            .set_assoc("y", [("a", "A"), ("b", ""), ("c", "C")])
            .expand();
        assert_eq!(left, "xa,A,b,,c,Cz");

        let left = uri_template
            .expander()
            .set_assoc("y", [] as [(&str, &str); 0])
            .expand();
        assert_eq!(left, "xz");
    }

    #[test]
    fn test_expand_named_operator() {
        let uri_template = UriTemplate::parse("x{?y}");

        let left = uri_template.expander().set_string("y", "Y").expand();
        assert_eq!(left, "x?y=Y");

        let left = uri_template.expander().set_string("y", "").expand();
        assert_eq!(left, "x?y=");

        let left = uri_template.expander().expand();
        assert_eq!(left, "x");

        let left = uri_template
            .expander()
            .set_list("y", ["A", "", "B"])
            .expand();
        assert_eq!(left, "x?y=A,,B");

        let left = uri_template
            .expander()
            .set_list("y", [] as [&str; 0])
            .expand();
        assert_eq!(left, "x");

        let left = uri_template
            .expander()
            .set_assoc("y", [("a", "A"), ("b", ""), ("c", "C")])
            .expand();
        assert_eq!(left, "x?y=a,A,b,,c,C");

        let left = uri_template
            .expander()
            .set_assoc("y", [] as [(&str, &str); 0])
            .expand();
        assert_eq!(left, "x");
    }

    #[test]
    fn test_explode_unnamed_operator() {
        let uri_template = UriTemplate::parse("x{/y*}");

        let left = uri_template.expander().set_string("y", "ABC").expand();
        assert_eq!(left, "x/ABC");

        let left = uri_template.expander().expand();
        assert_eq!(left, "x");

        let left = uri_template
            .expander()
            .set_list("y", ["A", "", "B"])
            .expand();
        assert_eq!(left, "x/A//B");

        let left = uri_template
            .expander()
            .set_list("y", [] as [&str; 0])
            .expand();
        assert_eq!(left, "x");

        let left = uri_template
            .expander()
            .set_assoc("y", [("a", "A"), ("b", ""), ("c", "C")])
            .expand();
        assert_eq!(left, "x/a=A/b=/c=C");

        let left = uri_template
            .expander()
            .set_assoc("y", [] as [(&str, &str); 0])
            .expand();
        assert_eq!(left, "x");
    }

    #[test]
    fn test_explode_named_operator() {
        let uri_template = UriTemplate::parse("x{;y*}");

        let left = uri_template.expander().set_string("y", "ABC").expand();
        assert_eq!(left, "x;y=ABC");

        let left = uri_template.expander().expand();
        assert_eq!(left, "x");

        let left = uri_template
            .expander()
            .set_list("y", ["A", "", "B"])
            .expand();
        assert_eq!(left, "x;y=A;y;y=B");

        let left = uri_template
            .expander()
            .set_list("y", [] as [&str; 0])
            .expand();
        assert_eq!(left, "x");

        let left = uri_template
            .expander()
            .set_assoc("y", [("a", "A"), ("b", ""), ("c", "C")])
            .expand();
        assert_eq!(left, "x;a=A;b;c=C");

        let left = uri_template
            .expander()
            .set_assoc("y", [] as [(&str, &str); 0])
            .expand();
        assert_eq!(left, "x");
    }
}
