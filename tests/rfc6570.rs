use uri_template::{UriTemplate, Value};

macro_rules! expand {
    ($template:expr, $right:expr) => {
        let uri_template = UriTemplate::parse($template);
        let variables = get_variables();
        let left = uri_template.expand(&variables);
        assert_eq!(left, $right);
    };
}

fn get_variables() -> Vec<(String, Value)> {
    [
        ("count", Value::from_list(["one", "two", "three"])),
        ("dom", Value::from_list(["example", "com"])),
        ("dub", Value::from_string("me/too")),
        ("hello", Value::from_string("Hello World!")),
        ("half", Value::from_string("50%")),
        ("var", Value::from_string("value")),
        ("who", Value::from_string("fred")),
        ("base", Value::from_string("http://example.com/home/")),
        ("path", Value::from_string("/foo/bar")),
        ("list", Value::from_list(["red", "green", "blue"])),
        (
            "keys",
            Value::from_assoc([("semi", ";"), ("dot", "."), ("comma", ",")]),
        ),
        ("v", Value::from_string("6")),
        ("x", Value::from_string("1024")),
        ("y", Value::from_string("768")),
        ("empty", Value::from_string("")),
        ("empty_keys", Value::List(vec![])),
    ]
    .iter()
    .cloned()
    .map(|(k, v)| (k.to_string(), v))
    .collect()
}

#[test]
fn simple_string_expansion() {
    expand!("{var}", "value");
    expand!("{hello}", "Hello%20World%21");
    expand!("{half}", "50%25");
    expand!("O{empty}X", "OX");
    expand!("O{undef}X", "OX");
    expand!("{x,y}", "1024,768");
    expand!("{x,hello,y}", "1024,Hello%20World%21,768");
    expand!("?{x,empty}", "?1024,");
    expand!("?{x,undef}", "?1024");
    expand!("?{undef,y}", "?768");
    expand!("{var:3}", "val");
    expand!("{var:30}", "value");
    expand!("{list}", "red,green,blue");
    expand!("{list*}", "red,green,blue");
    expand!("{keys}", "semi,%3B,dot,.,comma,%2C");
    expand!("{keys*}", "semi=%3B,dot=.,comma=%2C");
}

#[test]
fn reserved_expansion() {
    expand!("{+var}", "value");
    expand!("{+hello}", "Hello%20World!");
    expand!("{+half}", "50%25");
    expand!("{base}index", "http%3A%2F%2Fexample.com%2Fhome%2Findex");
    expand!("{+base}index", "http://example.com/home/index");
    expand!("O{+empty}X", "OX");
    expand!("O{+undef}X", "OX");
    expand!("{+path}/here", "/foo/bar/here");
    expand!("here?ref={+path}", "here?ref=/foo/bar");
    expand!("up{+path}{var}/here", "up/foo/barvalue/here");
    expand!("{+x,hello,y}", "1024,Hello%20World!,768");
    expand!("{+path,x}/here", "/foo/bar,1024/here");
    expand!("{+path:6}/here", "/foo/b/here");
    expand!("{+list}", "red,green,blue");
    expand!("{+list*}", "red,green,blue");
    expand!("{+keys}", "semi,;,dot,.,comma,,");
    expand!("{+keys*}", "semi=;,dot=.,comma=,");
}

#[test]
fn fragment_expansion() {
    expand!("{#var}", "#value");
    expand!("{#hello}", "#Hello%20World!");
    expand!("{#half}", "#50%25");
    expand!("foo{#empty}", "foo#");
    expand!("foo{#undef}", "foo");
    expand!("{#x,hello,y}", "#1024,Hello%20World!,768");
    expand!("{#path,x}/here", "#/foo/bar,1024/here");
    expand!("{#path:6}/here", "#/foo/b/here");
    expand!("{#list}", "#red,green,blue");
    expand!("{#list*}", "#red,green,blue");
    expand!("{#keys}", "#semi,;,dot,.,comma,,");
    expand!("{#keys*}", "#semi=;,dot=.,comma=,");
}

#[test]
fn label_expansion_with_dot_prefix() {
    expand!("{.who}", ".fred");
    expand!("{.who,who}", ".fred.fred");
    expand!("{.half,who}", ".50%25.fred");
    expand!("www{.dom*}", "www.example.com");
    expand!("X{.var}", "X.value");
    expand!("X{.empty}", "X.");
    expand!("X{.undef}", "X");
    expand!("X{.var:3}", "X.val");
    expand!("X{.list}", "X.red,green,blue");
    expand!("X{.list*}", "X.red.green.blue");
    expand!("X{.keys}", "X.semi,%3B,dot,.,comma,%2C");
    expand!("X{.keys*}", "X.semi=%3B.dot=..comma=%2C");
    expand!("X{.empty_keys}", "X");
    expand!("X{.empty_keys*}", "X");
}

#[test]
fn path_segment_expansion() {
    expand!("{/who}", "/fred");
    expand!("{/who,who}", "/fred/fred");
    expand!("{/half,who}", "/50%25/fred");
    expand!("{/who,dub}", "/fred/me%2Ftoo");
    expand!("{/var}", "/value");
    expand!("{/var,empty}", "/value/");
    expand!("{/var,undef}", "/value");
    expand!("{/var,x}/here", "/value/1024/here");
    expand!("{/var:1,var}", "/v/value");
    expand!("{/list}", "/red,green,blue");
    expand!("{/list*}", "/red/green/blue");
    expand!("{/list*,path:4}", "/red/green/blue/%2Ffoo");
    expand!("{/keys}", "/semi,%3B,dot,.,comma,%2C");
    expand!("{/keys*}", "/semi=%3B/dot=./comma=%2C");
}

#[test]
fn path_parameter_expansion() {
    expand!("{;who}", ";who=fred");
    expand!("{;half}", ";half=50%25");
    expand!("{;empty}", ";empty");
    expand!("{;v,empty,who}", ";v=6;empty;who=fred");
    expand!("{;v,bar,who}", ";v=6;who=fred");
    expand!("{;x,y}", ";x=1024;y=768");
    expand!("{;x,y,empty}", ";x=1024;y=768;empty");
    expand!("{;x,y,undef}", ";x=1024;y=768");
    expand!("{;hello:5}", ";hello=Hello");
    expand!("{;list}", ";list=red,green,blue");
    expand!("{;list*}", ";list=red;list=green;list=blue");
    expand!("{;keys}", ";keys=semi,%3B,dot,.,comma,%2C");
    expand!("{;keys*}", ";semi=%3B;dot=.;comma=%2C");
}

#[test]
fn form_style_query_expansion() {
    expand!("{?who}", "?who=fred");
    expand!("{?half}", "?half=50%25");
    expand!("{?x,y}", "?x=1024&y=768");
    expand!("{?x,y,empty}", "?x=1024&y=768&empty=");
    expand!("{?x,y,undef}", "?x=1024&y=768");
    expand!("{?var:3}", "?var=val");
    expand!("{?list}", "?list=red,green,blue");
    expand!("{?list*}", "?list=red&list=green&list=blue");
    expand!("{?keys}", "?keys=semi,%3B,dot,.,comma,%2C");
    expand!("{?keys*}", "?semi=%3B&dot=.&comma=%2C");
}

#[test]
fn form_style_query_continuation() {
    expand!("{&who}", "&who=fred");
    expand!("{&half}", "&half=50%25");
    expand!("?fixed=yes{&x}", "?fixed=yes&x=1024");
    expand!("{&x,y,empty}", "&x=1024&y=768&empty=");
    expand!("{&x,y,undef}", "&x=1024&y=768");
    expand!("{&var:3}", "&var=val");
    expand!("{&list}", "&list=red,green,blue");
    expand!("{&list*}", "&list=red&list=green&list=blue");
    expand!("{&keys}", "&keys=semi,%3B,dot,.,comma,%2C");
    expand!("{&keys*}", "&semi=%3B&dot=.&comma=%2C");
}
