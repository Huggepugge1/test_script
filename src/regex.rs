use crate::error::{ParseError, ParseErrorType};
use crate::token::Token;
use regex_syntax::hir;

fn expand_class(class: hir::ClassUnicode) -> Vec<String> {
    let mut result = Vec::new();
    for range in class.ranges().iter() {
        for c in range.start()..=range.end() {
            if c != '\n' && c.is_ascii() && (c.is_ascii_graphic() || c.is_ascii_whitespace()) {
                result.push(c.to_string());
            }
        }
    }
    result
}

fn parse_repetiton(
    hir: hir::Repetition,
    token: &Token,
    max: u32,
) -> Result<Vec<String>, ParseError> {
    let sub_class = parse_kind((hir.sub).into_kind(), token, max)?;
    let mut result: Vec<String> = Vec::new();
    let min = hir.min;
    let max = hir.max.unwrap_or(max);
    for i in min..=max {
        let combinations = itertools::Itertools::multi_cartesian_product(
            (0..i).map(|_| sub_class.iter().cloned()),
        );
        for combination in combinations.clone() {
            let joined = combination.join("");
            result.push(joined);
        }
    }
    Ok(result)
}

fn parse_concat(hirs: Vec<hir::Hir>, token: &Token, max: u32) -> Result<Vec<String>, ParseError> {
    let mut result = Vec::new();
    for hir in hirs {
        let mut sub_class = parse_kind(hir.into_kind(), token, max)?;
        if result.is_empty() {
            result.append(&mut sub_class);
        } else {
            let old_result = result.clone();
            result.clear();
            for i in old_result {
                for sub in sub_class.iter() {
                    let joined = format!("{}{}", i, sub);
                    result.push(joined);
                }
            }
        }
    }
    Ok(result)
}

fn parse_kind(kind: hir::HirKind, token: &Token, max: u32) -> Result<Vec<String>, ParseError> {
    match kind {
        hir::HirKind::Literal(hir) => Ok(vec![String::from_utf8_lossy(&hir.0).to_string()]),
        hir::HirKind::Class(hir) => match hir {
            hir::Class::Unicode(class) => Ok(expand_class(class)),
            hir::Class::Bytes(class) => Ok(expand_class(class.to_unicode_class().unwrap())),
        },
        hir::HirKind::Repetition(hir) => Ok(parse_repetiton(hir, token, max)?),
        hir::HirKind::Concat(hirs) => Ok(parse_concat(hirs, token, max)?),
        _hir => Err(ParseError::new(ParseErrorType::RegexError, token.clone())),
    }
}

pub fn parse(token: &Token, max: u32) -> Result<Vec<String>, ParseError> {
    let value = match &token.r#type {
        crate::token::TokenType::RegexLiteral { value } => value,
        _ => unreachable!(),
    };
    let kind = regex_syntax::parse(value).unwrap().into_kind();
    parse_kind(kind.clone(), token, max)
}
