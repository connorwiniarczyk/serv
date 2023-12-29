use super::*;

type Rule = Fn(&mut Cursor) -> Result<AstNode, MatchFail>;

fn either(cursor: &mut Cursor, rules: &[&Rule]) -> Result<AstNode, MatchFail> {
	let mut i = 1;
	let mut rules_iter = rules.iter();
	while let Some(rule) = rules_iter.next() {
		let mut test = cursor.clone(); 
		if let Ok(node) = rule(&mut test) { *cursor = test; return Ok(node) }
	}
	Err(MatchFail)
}

pub fn root(cursor: &mut Cursor) -> Result<AstNode, MatchFail> {
	let mut output = Vec::<AstNode>::new();
	loop {
		cursor.skip(&[' ', '\n', '\t']);
		if let Ok(route) = route(cursor) { output.push(route) };
		cursor.skip(&[' ', '\n', '\t']);
		_ = comment(cursor);
		if cursor.peek() == None { break; }
	}
	Ok(AstNode::Root(output))
}

fn comment(cursor: &mut Cursor) -> Result<(), MatchFail> {
	cursor.expect("#")?;
	while let Some(c) = cursor.next() {
		if c == '\n' { break };
	}
	Ok(())
}

fn route(start: &mut Cursor) -> Result<AstNode, MatchFail> {
	let mut cursor = start.clone();
	cursor.clone().expect("/")?;
	let pattern = pattern(&mut cursor)?;
	cursor.skip(&[' ', '\t']);
	cursor.expect("=>")?;
	cursor.skip(&[' ', '\t']);
	let expression = expression(&mut cursor)?;
	*start = cursor;
	Ok(AstNode::Route((Box::new(pattern), Box::new(expression))))
}

fn pattern(start: &mut Cursor) -> Result<AstNode, MatchFail> {
	let mut cursor = start.clone();
	let mut output = String::new();
	loop {
		match cursor.next() {
			Some(' ' | '\t') => { return Ok(AstNode::Pattern(output)) },
			Some(c @ _) => { output.push(c); start.next(); },
			None => { return Err(MatchFail); }
		}
	}
}

fn function(start: &mut Cursor) -> Result<AstNode, MatchFail> {
	let mut cursor = start.clone();
	cursor.expect("@")?;
	start.next();

	let name = cursor.consume_identifier()?;
	*start = cursor;
	Ok(AstNode::Function(name))
}

fn variable(start: &mut Cursor) -> Result<AstNode, MatchFail> {
	let mut cursor = start.clone();
	cursor.expect("$")?;
	start.next();

	let name = cursor.consume_identifier()?;
	*start = cursor;
	Ok(AstNode::Variable(name))
}

fn literal(start: &mut Cursor) -> Result<AstNode, MatchFail> {
	let mut cursor = start.clone();
	let mut output = String::new();
	loop {
		match cursor.next() {
			Some(' ' | '\t' | '\n') => { return Ok(AstNode::Text(output)) },
			Some(c @ _) => { output.push(c); start.next(); },
			None => { return Err(MatchFail); }
		}
	}
}

fn template(start: &mut Cursor) -> Result<AstNode, MatchFail> {
	let mut cursor = start.clone();
	cursor.expect("{")?;
	let mut output = Vec::<AstNode>::new();
	let mut buffer = String::new();
	loop {
		if cursor.peek() == Some('}') { break; }
		if cursor.clone().expect("@{").is_ok() {
			let mut new = cursor.clone();
			if let Ok(e) = inner_expression(&mut new) {
				output.push(AstNode::Text(buffer.clone()));
				output.push(e);
				buffer.clear();
				cursor = new;
			}
		}
		if cursor.clone().expect("$").is_ok() {
			let mut new = cursor.clone();
			if let Ok(e) = variable(&mut new) {
				output.push(AstNode::Text(buffer.clone()));
				output.push(e);
				buffer.clear();
				cursor = new;
			}
		}
		buffer.push(cursor.next().ok_or(MatchFail)?);
	}

	output.push(AstNode::Text(buffer));
	cursor.expect("}")?;
	*start = cursor;
	Ok(AstNode::Template(output))
}

fn inner_expression(start: &mut Cursor) -> Result<AstNode, MatchFail> {
	let mut cursor = start.clone();
	let mut output = Vec::<AstNode>::new();
	cursor.expect("@{")?;
	loop {
		cursor.skip(&[' ', '\t', '\n']);
		if cursor.peek() == Some('}') { break; }
		output.push(either(&mut cursor, &[&inner_expression, &function, &variable, &template, &literal])?);
	}

	cursor.expect("}")?;
	*start = cursor;
	Ok(AstNode::Expression(output))
}

fn expression(cursor: &mut Cursor) -> Result<AstNode, MatchFail> {
	let mut output = Vec::<AstNode>::new();
	loop {
		cursor.skip(&[' ', '\t']);
		if cursor.clone().expect("\n").is_ok() { return Ok(AstNode::Expression(output)) }
		output.push(either(cursor, &[&inner_expression, &function, &variable, &template, &literal])?);
	}
}


// fn path_segment_double_wildcard(cursor: &mut Cursor) -> Result<AstNode, MatchFail> {
//	cursor.expect("**")?;
//	Ok(AstNode::DeepWildcard(cursor.consume_path_segment()?))
// }

// fn path_segment_wildcard(cursor: &mut Cursor) -> Result<AstNode, MatchFail> {
//	cursor.expect("*")?;
//	Ok(AstNode::Wildcard(cursor.consume_path_segment()?))
// }

// fn path_segment_value(cursor: &mut Cursor) -> Result<AstNode, MatchFail> {
//	Ok(AstNode::Value(cursor.consume_path_segment()?))
// }

// fn path_segment(cursor: &mut Cursor) -> Result<AstNode, MatchFail> {
//	cursor.expect("/")?;
//	   Ok(AstNode::PathSegment(
//		   Box::new(either(cursor, &[
//			   &path_segment_double_wildcard,
//			   &path_segment_wildcard,
//			   &path_segment_value,
//		   ])?

//	   )))
// }

// fn path_extension(cursor: &mut Cursor) -> Result<AstNode, MatchFail> {
//	cursor.expect(".")?;
//	   Ok(AstNode::PathExtension(
//		   Box::new(either(cursor, &[
//			   &path_segment_double_wildcard,
//			   &path_segment_wildcard,
//			   &path_segment_value,
//		   ])?
//	   )))
// }

// fn pattern(cursor: &mut Cursor) -> Result<AstNode, MatchFail> {
//	let mut output = Vec::<AstNode>::new();
//	loop {
//		either(cursor, &[&path_segment, &path_extension]).and_then(|s: AstNode| {output.push(s); Ok(())} )?;
//		if let Ok(_) = cursor.expect(":") { break; }
//	}
//	Ok(AstNode::Pattern(output))
// }
