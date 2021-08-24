pub type PushAllow = fn(&mut String, &str);

pub fn push_literal(dst: &mut String, src: &str) {
    pct_encode(is_unreserved, dst, src);
}

pub fn push_allow_unreserved(dst: &mut String, src: &str) {
    src.chars().for_each(|c| push_char(is_unreserved, dst, c));
}

pub fn push_allow_unreserved_reserved(dst: &mut String, src: &str) {
    src.chars()
        .for_each(|c| push_char(is_unreserved_reserved, dst, c));
}

pub fn is_alpha(c: char) -> bool {
    matches!(c, '\x41'..='\x5A' | '\x61'..='\x7A')
}

pub fn is_digit(c: char) -> bool {
    matches!(c, '\x30'..='\x39')
}

pub fn is_hexdig(c: char) -> bool {
    is_digit(c)
        || matches!(c, 'A' | 'B' | 'C' | 'D' | 'E' | 'F')
        || matches!(c, 'a' | 'b' | 'c' | 'd' | 'e' | 'f')
}

pub fn is_unreserved(c: char) -> bool {
    is_alpha(c) || is_digit(c) || matches!(c, '-' | '.' | '_' | '~')
}

pub fn is_reserved(c: char) -> bool {
    is_gen_delims(c) || is_sub_delims(c)
}

pub fn is_gen_delims(c: char) -> bool {
    matches!(c, ':' | '/' | '?' | '#' | '[' | ']' | '@')
}

pub fn is_sub_delims(c: char) -> bool {
    matches!(
        c,
        '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '='
    )
}

pub fn is_unreserved_reserved(c: char) -> bool {
    is_reserved(c) || is_unreserved(c)
}

type IsAllowed = fn(char) -> bool;

enum PctEncodeState {
    S0,
    S1,
    S2(char),
}

impl PctEncodeState {
    fn transition(self, is_allowed: IsAllowed, dst: &mut String, c: char) -> Self {
        match self {
            PctEncodeState::S1 if is_hexdig(c) => PctEncodeState::S2(c),
            PctEncodeState::S2(b) if is_hexdig(c) => {
                dst.push('%');
                dst.push(b);
                dst.push(c);
                PctEncodeState::S0
            }
            _ => {
                self.push_incomplete(is_allowed, dst);
                if '%' == c {
                    PctEncodeState::S1
                } else {
                    push_char(is_allowed, dst, c);
                    PctEncodeState::S0
                }
            }
        }
    }

    fn push_incomplete(&self, is_allowed: IsAllowed, dst: &mut String) {
        match self {
            PctEncodeState::S0 => {}
            PctEncodeState::S1 => {
                dst.push_str("%25");
            }
            PctEncodeState::S2(c) => {
                dst.push_str("%25");
                push_char(is_allowed, dst, *c);
            }
        }
    }
}

fn pct_encode(is_allowed: IsAllowed, dst: &mut String, src: &str) {
    let mut state = PctEncodeState::S0;
    for c in src.chars() {
        state = state.transition(is_allowed, dst, c);
    }
    state.push_incomplete(is_allowed, dst);
}

fn push_char(is_allowed: IsAllowed, dst: &mut String, c: char) {
    if is_allowed(c) {
        dst.push(c);
    } else {
        push_hex_char(dst, c);
    }
}

fn push_hex_char(dst: &mut String, c: char) {
    let mut buf = [0; 4];
    let s = c.encode_utf8(&mut buf);
    s.as_bytes().iter().for_each(|b| push_hex_u8(dst, *b));
}

const HEX_DIGITS: &[u8] = b"0123456789ABCDEF";

fn push_hex_u8(dst: &mut String, b: u8) {
    let hi = char::from(HEX_DIGITS[usize::from(b >> 4)]);
    let lo = char::from(HEX_DIGITS[usize::from(b & 0xF)]);
    dst.push('%');
    dst.push(hi);
    dst.push(lo);
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test {
        ($s:expr, $right:expr) => {
            let mut left = String::new();
            pct_encode(|_| true, &mut left, $s);
            assert_eq!(left, $right);
        };
    }

    #[test]
    fn test_pct_encode() {
        test!("Hello%20World!", "Hello%20World!");

        test!("", "");
        test!("%", "%25");
        test!("%2", "%252");
        test!("%2x", "%252x");
        test!("%20", "%20");
        test!("%20x", "%20x");
        test!("%20a", "%20a");

        test!("%%", "%25%25");
        test!("%%x", "%25%25x");
        test!("%%2", "%25%252");
        test!("%%2x", "%25%252x");
        test!("%%20", "%25%20");
        test!("%%20x", "%25%20x");
        test!("%%20a", "%25%20a");

        test!("x", "x");
        test!("x%", "x%25");
        test!("x%x", "x%25x");
        test!("x%2", "x%252");
        test!("x%2x", "x%252x");
        test!("x%20", "x%20");
        test!("x%20x", "x%20x");
        test!("x%20a", "x%20a");

        test!("a", "a");
        test!("a%", "a%25");
        test!("a%x", "a%25x");
        test!("a%2", "a%252");
        test!("a%2x", "a%252x");
        test!("a%20", "a%20");
        test!("a%20x", "a%20x");
        test!("a%20a", "a%20a");
    }
}
