use crate::ServValue;
use crate::ServResult;
use crate::ServFunction;
use crate::Scope;
use crate::Words;
use crate::parser;
use crate::compile;
use crate::VecDeque;

pub fn count(input: ServValue, scope: &Scope) -> ServResult {
    let max = input.expect_int()?;
    let mut output = VecDeque::new();
    let mut i: i64 = 0;

    while i < max {
        output.push_back(ServValue::Int(i));
		i += 1;
    }

	Ok(ServValue::List(output))
}

pub fn choose(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let next = words.next().ok_or("not enought arguments")?;
    let arg = scope.get(&next).ok_or("not found")?.call(input.clone(), scope)?;
    let rest = words.eval(input, scope)?;

	let ServValue::List(list) = rest else { return Err("not a valid list") };

	let index: usize = arg.expect_int()?.try_into().unwrap();

	Ok(list[index.clamp(0, list.len() - 1)].clone())
}

pub fn get(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let arg = words.take_next(scope)?;
    let mut input = words.eval(input, scope)?;

    let output = match (arg.ignore_metadata(), input.ignore_metadata()) {
        (ServValue::Text(ref key), ServValue::Table(mut map)) => map.remove(key).ok_or("key not found")?,
        (ServValue::Int(index),    ServValue::List(mut list)) => list.remove(index.try_into().map_err(|e| "invalid index")?).ok_or("index not found")?,
        (_, _) => return Err("invalid key"),
    };

	Ok(output)
}

pub fn switch(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {

    let arg1 = words.next().unwrap();
    // let arg1 = (scope)?.expect_int()?;
    let arg2 = words.next().ok_or("not enough arguments")?;
    let ServFunction::List(options) = scope.get(&arg2).ok_or("not found")? else { return Err("switch needs its second arg to be a list literal") };

    let rest = words.eval(input, scope)?;
    let index: usize = scope.get(&arg1).unwrap().call(rest.clone(), scope)?.expect_int()?.try_into().unwrap();

	// let index: usize = arg1.try_into().unwrap();
    let choice = options[index.clamp(0, options.len() - 1)].clone();

    let output = scope.get(&choice).ok_or("not found")?.call(rest, scope)?;

	Ok(output)
}

pub fn sum(input: ServValue, scope: &Scope) -> ServResult {
    let ServValue::List(list) = input.ignore_metadata() else { return Err("sum needs to operate on a list") };
    let mut iter = list.into_iter().filter(|x| !matches!(x, ServValue::None)).peekable();

    let output: ServValue = match iter.peek().ok_or("tried to sum an empty list")? {
        ServValue::Int(i) => ServValue::Int(iter.map(|x| x.expect_int().unwrap()).sum()),
        ServValue::Text(t) => {
            let mut output = String::new();
            for element in iter {
                output.push_str(element.to_string().as_str());
            }
            ServValue::Text(output)
        },
        _ => return Err("please don't try to sum that"),
    };

	Ok(output)
}

pub fn join(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let intersperse = words.take_next(scope)?.to_string();
    let ServValue::List(list) = words.eval(input, scope)?.ignore_metadata() else { return Err("sum needs to operate on a list") };
    let t: Vec<String> = list.into_iter().map(|x| x.to_string()).collect();

    Ok(ServValue::Text(t.join(&intersperse)))
}

pub fn map(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let next = words.next().ok_or("not enough arguments")?;
    let arg = scope.get(&next).ok_or("not found")?;
    let rest = words.eval(input, scope)?;

	let mapped = match rest {
    	ServValue::List(list) => ServValue::List(list.into_iter().map(|a| arg.call(a, scope).unwrap()).collect()),
    	_ => todo!(),
	};

	Ok(mapped)
}

pub fn fold(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let next = words.next().ok_or("not enough arguments")?;
    let arg = scope.get(&next).ok_or("not found")?;
    let rest = words.eval(input, scope)?;

    let mut acc = ServValue::None;

    let ServValue::List(items) = rest else { return Err("not a list") };

    for (index, item) in items.into_iter().enumerate() {
        let mut child_scope = scope.make_child();
        child_scope.insert_name("acc",   ServFunction::Literal(acc.clone()));
        child_scope.insert_name("index", ServFunction::Literal(ServValue::Int(index.try_into().unwrap())));
        acc = arg.call(item, &child_scope)?;
    }

	Ok(acc)
}


pub fn split(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let arg = words.take_next(scope)?.to_string();
    let rest = words.eval(input, scope)?.to_string();

    Ok(ServValue::List(rest.split(&arg).map(|x| ServValue::Text(x.to_owned())).collect()))
}
