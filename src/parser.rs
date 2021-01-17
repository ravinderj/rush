use nom::{
  branch::alt,
  bytes::complete::{
    escaped, escaped_transform, tag, take_until, take_while, take_while1,
  },
  character::complete::{alphanumeric1 as alphanumeric, char, one_of, space1},
  combinator::{cut, map, verify},
  error::{context, ErrorKind, ParseError},
  sequence::{preceded, terminated},
  IResult,
};

#[derive(Debug, PartialEq)]
pub enum Token<'a> {
  Var(&'a str),
  Literal(&'a str),
  Substitution(&'a str, Vec<Token<'a>>),
  Word(std::string::String), // Need to think of some other name
  Pipe,
}

fn sp<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
  let chars = " \t\r\n";
  take_while(move |c| chars.contains(c))(i)
}

fn is_valid_var_char(c: char) -> bool {
  match c {
    '0'..='9' | 'a'..='z' | 'A'..='Z' | '_' => true,
    _ => false,
  }
}

fn var<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, Token<'a>, E> {
  context(
    "var",
    map(
      preceded(char('$'), take_while1(is_valid_var_char)),
      Token::Var,
    ),
  )(i)
}

fn _recognise_escaped_space<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> nom::IResult<&'a str, &'a str, E> {
  preceded(nom::character::complete::char('\\'), sp)(i)
}

fn escaped_space<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> nom::IResult<&'a str, &'a str, E> {
  nom::combinator::recognize(tag(" "))(i)
}

fn escaped_tab<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> nom::IResult<&'a str, &'a str, E> {
  nom::combinator::value("  ", tag("t"))(i)
}

fn escaped_backslash<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> nom::IResult<&'a str, &'a str, E> {
  nom::combinator::recognize(nom::character::complete::char('\\'))(i)
}

fn _alphanum<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> nom::IResult<&'a str, &'a str, E> {
  take_while1(is_valid_var_char)(i)
}

// fn word<'a, E: ParseError<&'a str>>(
//   i: &'a str,
// ) -> IResult<&'a str, Token<'a>, E> {
//   context(
//     "word",
//     map(
//       alt((alphanum, recognise_escaped_space)),
//       Token::Word,
//     ),
//   )(i)
// }

fn word<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, Token<'a>, E> {
  context(
    "word",
    map(
      // verify(escaped(alphanum, '\\', char('a')), |s: &str| s != ""),
      // escaped(alphanum, '\\', char('a')),
      verify(
        escaped_transform(
          alphanumeric,
          '\\',
          alt((escaped_space, escaped_tab, escaped_backslash)),
        ),
        |s: &str| s != "",
      ),
      Token::Word,
    ),
  )(i)
}

fn literal<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, Token<'a>, E> {
  map(preceded(char('\"'), terminated(take_until("\""), char('\"'))), Token::Literal)(i)
}

fn _parse_str<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, &'a str, E> {
  escaped(alt((alphanumeric, space1)), '\\', one_of("\"n\\"))(i)
}

fn _string<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, &'a str, E> {
  context(
    "string",
    preceded(char('\"'), cut(terminated(_parse_str, char('\"')))),
  )(i)
}

fn pipe<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, Token<'a>, E> {
  context("pipe", map(char('|'), |_| Token::Pipe))(i)
}

fn segment<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, Token<'a>, E> {
  context("segment", preceded(sp, alt((word, literal, pipe, var))))(i) // this seems wrong
}

fn root<'a, E: ParseError<&'a str>>(
  i: &'a str,
) -> IResult<&'a str, Vec<Token<'a>>, E> {
  map(segment, |tuple_vec| vec![tuple_vec])(i)
}

pub fn parse(i: &str) -> Vec<Token> {
  //this seems wrong
  let mut all_tokens: Vec<Token> = Vec::new();
  let mut new_i = i.clone();
  loop {
    match root::<(&str, ErrorKind)>(new_i) {
      Ok((remaining, tokens)) => {
        all_tokens.extend(tokens);
        println!("{:?}", remaining);
        if remaining.len() == 0 {
          break;
        }
        new_i = remaining;
      }
      Err(e) => {
        println!("{:?}", e);
        break;
      }
    }
  }
  all_tokens
}

////////////////////////////////////////////////////////////////////////////////////////////////////
//*****Tests**************************************************************************************//
////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
pub mod tests {
  use crate::parser::Token::Literal;
  use crate::parser::Token::Pipe;
  use crate::parser::Token::Var;
  use crate::parser::Token::Word;
  use crate::parser::{parse, Token};

  #[test]
  fn should_parse_empty_string() {
    let actual = parse("");
    let expected: Vec<Token> = vec![];
    assert_eq!(expected, actual)
  }

  #[test]
  fn should_parse_multiple_strings() {
    let actual = parse(r#""hello" "world""#);
    let expected: Vec<Token> = vec![Literal("hello"), Literal("world")];
    assert_eq!(actual, expected)
  }

  #[test]
  fn should_parse_pipe() {
    let actual = parse("\"hello\" | ");
    let expected: Vec<Token> = vec![Literal("hello"), Pipe];
    assert_eq!(actual, expected)
  }

  #[test]
  fn should_parse_word_with_string() {
    let actual = parse(r#"echo "hello world""#);
    let expected: Vec<Token> =
      vec![Word("echo".to_string()), Literal("hello world")];
    assert_eq!(expected, actual)
  }

  #[test]
  fn should_parse_word_with_pipe() {
    let actual = parse("echo hello | cat");
    let expected: Vec<Token> = vec![
      Word("echo".to_string()),
      Word("hello".to_string()),
      Pipe,
      Word("cat".to_string()),
    ];
    assert_eq!(expected, actual)
  }

  #[test]
  fn should_parse_word_with_string_and_pipe() {
    let actual = parse(r#"ls | grep "catch me if you can""#);
    let expected: Vec<Token> = vec![
      Word("ls".to_string()),
      Pipe,
      Word("grep".to_string()),
      Literal("catch me if you can"),
    ];
    assert_eq!(expected, actual)
  }

  #[test]
  fn should_parse_word_with_escaped_space_string() {
    let actual = parse("echo hello\\ world ");
    let expected: Vec<Token> =
      vec![Word("echo".to_string()), Word("hello world".to_string())];
    assert_eq!(expected, actual)
  }

  #[test]
  fn should_parse_word_with_escaped_tab_string() {
    let actual = parse("echo hello\\tworld ");
    let expected: Vec<Token> =
      vec![Word("echo".to_string()), Word("hello  world".to_string())];
    assert_eq!(expected, actual)
  }

  #[test]
  fn should_parse_word_with_escaped_slash_string() {
    let actual = parse("echo hello\\\\world");
    let expected: Vec<Token> =
      vec![Word("echo".to_string()), Word("hello\\world".to_string())];
    assert_eq!(expected, actual)
  }

  #[test]
  fn should_parse_word_joined_strings() {
    let actual = parse("echo \"hello\"\"world\"");
    let expected: Vec<Token> =
      vec![Word("echo".to_string()), Literal("hello"), Literal("world")];
    assert_eq!(expected, actual)
  }

  #[test]
  fn should_parse_variable() {
    let actual = parse(r#"$HOME"#);
    let expected: Vec<Token> = vec![Var("HOME")];
    assert_eq!(actual, expected)
  }

  #[test]
  fn should_parse_multiple_variables() {
    let actual = parse(r#"$HOME$SHELL"#);
    let expected: Vec<Token> = vec![Var("HOME"), Var("SHELL")];
    assert_eq!(actual, expected)
  }

  #[test]
  fn should_parse_multiple_separated_variables() {
    let actual = parse(r#""$HOME-6435745%^&^%^$SHELL""#);
    let expected: Vec<Token> = vec![Literal("$HOME-6435745%^&^%^$SHELL")];
    assert_eq!(actual, expected)
  }
}
