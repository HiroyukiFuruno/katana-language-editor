use std::iter::Peekable;
use std::str::Chars;

pub(super) fn code_only(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '/' if chars.peek() == Some(&'/') => {
                skip_line_comment(&mut chars, &mut output);
            }
            '/' if chars.peek() == Some(&'*') => {
                skip_block_comment(&mut chars, &mut output);
            }
            '"' => {
                skip_string(&mut chars, &mut output);
            }
            _ => output.push(ch),
        }
    }
    output
}

fn skip_line_comment(chars: &mut Peekable<Chars<'_>>, output: &mut String) {
    chars.next();
    for comment in chars.by_ref() {
        if comment != '\n' {
            continue;
        }
        output.push('\n');
        break;
    }
}

fn skip_block_comment(chars: &mut Peekable<Chars<'_>>, output: &mut String) {
    chars.next();
    let mut previous = '\0';
    for comment in chars.by_ref() {
        if previous == '*' && comment == '/' {
            break;
        }
        previous = comment;
        if comment == '\n' {
            output.push('\n');
        }
    }
}

fn skip_string(chars: &mut Peekable<Chars<'_>>, output: &mut String) {
    output.push(' ');
    let mut escaped = false;
    for string_char in chars.by_ref() {
        if string_char == '\n' {
            output.push('\n');
            continue;
        }
        if !escaped && string_char == '"' {
            break;
        }
        escaped = !escaped && string_char == '\\';
        output.push(' ');
    }
}
