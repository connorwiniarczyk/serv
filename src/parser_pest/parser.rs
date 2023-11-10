
use pest::*;
use crate::template::Template;

use crate::routetree::RouteTree;
use crate::routetree::PathNode;

use std::io::Read;

use pest::iterators::Pair;

use crate::ast::AstNode;

#[derive(Parser)]
#[grammar = "parser.pest"]
pub struct ServParser;

pub type Tree<'a> = Pair<'a, Rule>;
pub type ParseError = pest::error::Error<Rule>;

pub trait FromSyntax {
	fn from_syntax(input: Tree) -> Result<Self, ParseError> where Self: Sized;
}

impl FromSyntax for PathNode {
	fn from_syntax(input: Tree) -> Result<Self, ParseError> {

		let output = match input.into_inner().next() {
			None => PathNode::Literal(String::new()),
			Some(inner) => {
				let rule = inner.as_rule();
				let value = inner.into_inner().flatten()
					.find(|p| p.as_rule() == Rule::ident)
					.unwrap()
					.as_str();

				match rule {
					Rule::value => PathNode::Literal(value.to_owned()),
					Rule::wildcard => PathNode::Wildcard(value.to_owned()),
					Rule::double_wildcard => PathNode::Doublewildcard(value.to_owned()),
					_ => panic!(),
				}
			},
		};

		Ok(output)
	}
}


/// This is a special version of the parser written only for testing. It does not parse the
/// expression half of each route and instead stores an id where the expression would be to allow
/// for more convenient testing
#[cfg(test)]
pub fn parse_str_id_only(input: &str) -> Result<RouteTree<i32>, &'static str> {
	let mut syntax_tree = ServParser::parse(Rule::file, input).expect("parse error");
	let root = syntax_tree.next().unwrap();

	let mut output: RouteTree<i32> = RouteTree::new();
	let mut index = 0;

	let mut routes = root.into_inner();
	while let Some(route) = routes.next() {
		let mut route_inner = route.into_inner();
		let pattern = route_inner.next().unwrap();

		let mut nodes = pattern.into_inner().map(PathNode::from_syntax);

		let nodes_vec: Result<Vec<PathNode>, ParseError> = nodes.collect();
		output.insert(&mut nodes_vec.unwrap().iter(), index);
		index += 1;
	}
	
	return Ok(output)
}

// pub fn parse_str(input: &str) -> Result<>

pub fn parse<R: Read>(mut input: R) -> Result<RouteTree<AstNode>, &'static str> {
	let mut input_string = String::new();
	input.read_to_string(&mut input_string);
	let mut syntax_tree = ServParser::parse(Rule::file, &input_string)
	   .expect("parse error")
       .next()
	   .unwrap();

	let mut output: RouteTree<AstNode> = RouteTree::new();

	for route in syntax_tree.into_inner().filter(|x| x.as_rule() == Rule::route) {
	   let mut route_elements_iter = route.into_inner();
	   let pattern_syntax = route_elements_iter.next().unwrap();
	   let action_syntax = route_elements_iter.next().unwrap();
	   let expression_syntax = action_syntax.into_inner().next().unwrap();

	   let mut nodes = pattern_syntax.into_inner().map(PathNode::from_syntax);
	   let nodes_vec: Result<Vec<PathNode>, ParseError> = nodes.collect();

	   // let pattern = PathNode::from_syntax(pattern_syntax).expect("invalid pattern");
	   let action = AstNode::from_syntax(expression_syntax).expect("invalid expression");

	   output.insert(&mut nodes_vec.unwrap().iter(), action);

	}

	Ok(output)
}

use std::collections::HashMap;

use std::convert::TryInto;

impl AstNode {
	fn insert(&mut self, expression: Tree) -> Result<(), ()> {

		let syntax = expression.into_inner().next().unwrap();

		*self = match syntax.as_rule() {
			Rule::prefix => {
				let mut parts = syntax.into_inner();
				let operator = parts.next().ok_or(())?;
				let operand = parts.next();

				let value = match operand {
					Some(o) => {
						let mut x = AstNode::Placeholder;
						x.insert(o);
						Some(Box::new(x))
					},
					None => None,
				};

				AstNode::Prefix {
					op: operator.as_str().try_into().unwrap(),
					options: HashMap::new(),
					value: value,
				}
			},

			Rule::template => AstNode::Template(Template::from_syntax(syntax).unwrap()),

			rule @ _ => {
				panic!("invalid rule: {:?}", rule);
			},
		};

		Ok(())
	}
}

impl FromSyntax for AstNode {
	fn from_syntax(mut input: Tree) -> Result<Self, ParseError> {

		let mut output: AstNode = AstNode::Placeholder;
		output.insert(input);

		Ok(output)
	}
}

fn unindent(input: &str) -> String {
    fn count_spaces(line: &str) -> Option<usize> {
        for (i, ch) in line.chars().enumerate() {
            if ch != ' ' && ch != '\t' { return Some(i) }
        }
        return None
    }

    let mut lines = input.lines();
    let spaces = lines
        .clone()
        .skip(1)
        .filter_map(count_spaces)
        .min()
        .unwrap_or(0);

    // println!("{}", spaces);

    let mut output = Vec::with_capacity(input.len());
    for line in lines {
        if line.len() > spaces {
            output.extend_from_slice(&line.as_bytes()[spaces..]);
        } else {
            output.extend_from_slice(&line.as_bytes());
        }
        output.push(b'\n');
    }

    let out = String::from_utf8(output).unwrap();
    // println!("{}", out);
    out

}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	#[ignore]
	fn test() {
		let test = r#"/one:{test}
	/two:{abcd}
	/one/two:{abcd}
	/one/*two:{abcd}
	/one/*two/three:{abcd}
"#;
		let tree = parse_str_id_only(test).expect("error");
		assert_eq!(tree.get("/one"), Some(&0));
		assert_eq!(tree.get("/two"), Some(&1));
		assert_eq!(tree.get("/one/two"), Some(&2));
		assert_eq!(tree.get("/one/anything"), Some(&3));
		assert_eq!(tree.get("/one/anything/three"), Some(&4));
		println!("{:#?}", tree);
	}


	#[test]
	fn test_expression() {
		let input = "@html{abcd}";
		let mut syntax = ServParser::parse(Rule::expression, input).expect("parse error").next().unwrap();
		let mut node = AstNode::Placeholder;
		println!("{:#?}", syntax);

		node.insert(syntax);
	}
}
