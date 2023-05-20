
use pest::*;
use crate::route_table::RouteTable;
use std::io::Read;

use pest::iterators::Pair;

use crate::pattern::{Pattern, Node};

#[derive(Parser)]
#[grammar = "parser.pest"]
struct ServParser;

type Tree<'a> = Pair<'a, Rule>;
type ParseError = pest::error::Error<Rule>;

trait FromSyntax {
    fn from_syntax(input: Tree) -> Result<Self, ParseError> where Self: Sized;
}

impl FromSyntax for Node {
    fn from_syntax(input: Tree) -> Result<Self, ParseError> {

        let inner = input.clone().into_inner().next().unwrap().as_str();

        let output = match input.as_rule() {
            Rule::value => Self::Value(input.as_str().to_owned()),
            Rule::wildcard => Self::Variable(inner.to_owned()),
            Rule::double_wildcard => Self::Rest(inner.to_owned()),
            _ => unreachable!(),
        };
        
        Ok(output)
    }
}

impl FromSyntax for Pattern {
    fn from_syntax(input: Tree) -> Result<Self, ParseError> {
        let mut path: Vec<Node> = vec![];
        let mut extension: Option<Node> = None;
        for a in input.into_inner() {
            match a.as_rule() {
                Rule::path_node => {
                    path.push(Node::from_syntax(a.into_inner().next().unwrap()).unwrap());
                },
                Rule::extension => {
                    extension = Some(Node::from_syntax(a.into_inner().next().unwrap()).unwrap());
                },
                _ => todo!(),
            }
        }

        let output = Self { name: None, path, extension };
        Ok(output)
    }
}

pub fn parse<R: Read>(mut input: R) -> Result<RouteTable, &'static str> {
    let mut input_string = String::new();
    input.read_to_string(&mut input_string);
    let syntax_tree = ServParser::parse(Rule::file, &input_string)
        .expect("parse error")
        .next()
        .unwrap();

    for route in syntax_tree.into_inner() {
        println!("{:?}", route.as_str());

        let pattern_syntax = route.into_inner().next().unwrap();
        let pattern = Pattern::from_syntax(pattern_syntax);

        println!("{:?}", pattern);
    }
    
    todo!();
}
