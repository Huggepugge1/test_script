use regex_syntax::hir;

fn expand_class(class: hir::ClassUnicode) -> Vec<char> {
    let mut result = Vec::new();
    for range in class.ranges().iter() {
        for c in range.start()..=range.end() {
            if c != '\n' && c.is_ascii() && (c.is_ascii_graphic() || c.is_ascii_whitespace()) {
                result.push(c);
            }
        }
    }
    result
}

fn parse_kind(kind: hir::HirKind, max: u32) -> Vec<String> {
    match kind {
        hir::HirKind::Literal(hir) => vec![String::from_utf8_lossy(&hir.0).to_string()],
        hir::HirKind::Class(hir) => match hir {
            hir::Class::Unicode(class) => {
                let class = expand_class(class);
                let mut result = Vec::new();
                for c in class {
                    result.push(c.to_string());
                }
                result
            }
            hir::Class::Bytes(class) => {
                let class = expand_class(class.to_unicode_class().unwrap());
                let mut result = Vec::new();
                for c in class {
                    result.push(c.to_string());
                }
                result
            }
        },
        hir::HirKind::Repetition(hir) => {
            let sub_class = parse_kind((hir.sub).into_kind(), max);
            let mut result: Vec<String> = Vec::new();
            let min = hir.min;
            let max = hir.max.unwrap_or(max);
            for i in min..=max {
                let combinations = itertools::Itertools::multi_cartesian_product(
                    (0..i).map(|_| sub_class.iter().cloned()),
                );
                for combination in combinations.clone() {
                    let joined = combination
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .join("");
                    result.push(joined);
                }
            }
            result
        }
        hir::HirKind::Concat(hirs) => {
            let mut result = Vec::new();
            for hir in hirs {
                let mut sub_class = parse_kind(hir.into_kind(), max);
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
            result
        }
        hir => panic!("Unsupported kind {:?}", hir),
    }
}

pub fn parse(regex: &str, max: u32) -> Vec<String> {
    let kind = regex_syntax::parse(regex).unwrap().into_kind();
    parse_kind(kind.clone(), max)
}
