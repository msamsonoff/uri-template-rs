#![allow(dead_code, unused_imports, unused_variables)]

use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use indexmap::IndexMap;
use serde::Deserialize;
use serde_json::{from_reader, Number};

use uri_template::{Expander, UriTemplate, Value, Variables};

#[derive(Deserialize)]
#[serde(untagged)]
enum VariableValue {
    Number(Number),
    String(String),
    Array(Vec<String>),
    Object(IndexMap<String, String>),
}

#[derive(Deserialize)]
struct Group {
    #[serde(default = "default_level")]
    level: u32,
    variables: IndexMap<String, VariableValue>,
    testcases: Vec<(String, serde_json::Value)>,
}

fn default_level() -> u32 {
    4
}

impl<'a> Variables<'a, Value> for Group {
    fn get(&self, k: &str) -> Option<uri_template::Value> {
        self.variables
            .get(k)
            .map(|v| match v {
                VariableValue::Number(n) => Some(Value::from_string(n.to_string())),
                VariableValue::String(s) => Some(Value::from_string(s)),
                VariableValue::Array(a) => Some(Value::from_list(a)),
                VariableValue::Object(o) => Some(Value::from_assoc(o)),
            })
            .flatten()
    }
}

fn uritemplate_test<P>(path: P) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
{
    let f = File::open(path)?;
    let r = BufReader::new(f);
    let m: IndexMap<String, Group> = from_reader(r)?;
    for g in m.values() {
        for (t, v) in &g.testcases {
            let uri_template = UriTemplate::parse(t);
            let left = uri_template.expand(g);
            match v {
                serde_json::Value::String(right) => {
                    assert_eq!(left, *right);
                }
                serde_json::Value::Array(right) => {
                    let option = right.iter().find(|&r| match r {
                        serde_json::Value::String(s) => *s == left,
                        _ => false,
                    });
                    assert!(option.is_some());
                }
                serde_json::Value::Bool(false) => {
                    // TODO: this one is complicated since there are both parse failures and expand
                    // failures
                    //
                    // also i checked a bunch of other implementations and they all do wildly
                    // different things
                    //
                    // and what to do about prefix applied to assoc/list?
                    // https://github.com/uri-templates/uritemplate-test/pull/27#issuecomment-25305841
                }
                _ => {
                    Err("invalid JSON")?;
                }
            }
        }
    }
    Ok(())
}

#[test]
#[ignore]
fn test_spec_examples() -> Result<(), Box<dyn Error>> {
    uritemplate_test("tests/uritemplate-test/spec-examples.json")
}

#[test]
#[ignore]
fn test_extended_tests() -> Result<(), Box<dyn Error>> {
    uritemplate_test("tests/uritemplate-test/extended-tests.json")
}

#[test]
#[ignore]
fn test_negative_tests() -> Result<(), Box<dyn Error>> {
    uritemplate_test("tests/uritemplate-test/negative-tests.json")
}
