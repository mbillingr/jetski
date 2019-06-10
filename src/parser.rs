use crate::error::{ErrorKind, Result};
use crate::object::{ListBuilder, Object};
use pest::{iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "r7rs.pest"]
pub struct R7rsGrammar;

pub fn parse_datum(input: &str) -> Result<Object> {
    let mut datum = R7rsGrammar::parse(Rule::datum, input)?;
    walk_datum(datum.next().unwrap())
}

fn walk_datum(pair: Pair<Rule>) -> Result<Object> {
    match pair.as_rule() {
        Rule::list => walk_list(pair),
        Rule::number => walk_number(pair),
        Rule::symbol => walk_symbol(pair),
        Rule::string_content => walk_string(pair),
        _ => unimplemented!(),
    }
}

fn walk_list(pair: Pair<Rule>) -> Result<Object> {
    let mut parse_list = pair.into_inner();
    let mut list_builder = ListBuilder::new();
    while let Some(list_item) = parse_list.next() {
        if list_item.as_rule() == Rule::dot {
            let item = walk_datum(parse_list.next().unwrap())?;
            list_builder.set_cdr(item);
        } else {
            let item = walk_datum(list_item)?;
            list_builder.append(item);
        }
    }
    Ok(list_builder.build())
}

fn walk_number(pair: Pair<Rule>) -> Result<Object> {
    let number = pair.into_inner().next().unwrap();
    match number.as_rule() {
        Rule::num_2 => walk_num_with_radix(number, 2),
        Rule::num_8 => walk_num_with_radix(number, 8),
        Rule::num_10 => walk_num_with_radix(number, 10),
        Rule::num_16 => walk_num_with_radix(number, 16),
        _ => unreachable!(),
    }
}

fn walk_num_with_radix(pair: Pair<Rule>, radix: u32) -> Result<Object> {
    let mut inner = pair.clone().into_inner();
    let exactness = inner.next().unwrap();
    let value = inner.next().unwrap();
    let integer_result = i64::from_str_radix(value.as_str(), radix);
    match (exactness.as_rule(), integer_result) {
        (Rule::exact, Ok(i)) | (Rule::empty, Ok(i)) => Ok(Object::integer(i)),
        (Rule::exact, Err(_)) => {
            Err(ErrorKind::InvalidNumericConstant(pair.as_str().to_owned()).into())
        }
        (Rule::inexact, Ok(i)) => Ok(Object::float(i as f64)),
        (Rule::inexact, Err(_)) | (Rule::empty, Err(_)) => value
            .as_str()
            .parse::<_>()
            .map(Object::float)
            .map_err(|_| ErrorKind::InvalidNumericConstant(pair.as_str().to_owned()).into()),
        _ => unreachable!(),
    }
}

fn walk_symbol(pair: Pair<Rule>) -> Result<Object> {
    let identifier = pair.into_inner().next().unwrap();
    match identifier.as_rule() {
        Rule::delimited_identifier | Rule::normal_identifier => {
            Ok(Object::symbol(identifier.as_str()))
        }
        _ => unreachable!(),
    }
}

fn walk_string(pair: Pair<Rule>) -> Result<Object> {
    Ok(Object::string(pair.as_str().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        println!("{:?}", R7rsGrammar::parse(Rule::prefix_10, "#i"));
        println!(
            "{:?}",
            parse_datum("(#e1 |x y| #i2 \"foo\" bar #b10 4.7 . 5)").unwrap()
        );
        println!(
            "{:?}",
            parse_datum("(define (two-sqr x) (* 2 x x))").unwrap()
        );
    }
}
