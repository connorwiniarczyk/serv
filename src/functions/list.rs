use crate::ServValue;
use crate::ServResult;
use crate::Stack;
use crate::servparser;
use crate::VecDeque;
use crate::{Label, ServFn};
use std::collections::HashMap;


fn take(mut input: VecDeque<ServValue>, scope: &mut Stack) -> ServResult {
    let arg  = input.pop_front().expect("word expected");
   	let ServValue::Ref(name @ Label::Name(_)) = arg else { panic!("'<' expects a ref") };

   	let rest = crate::value::eval(input, scope)?;

   	let out = match rest {
       	ServValue::List(mut l) => { scope.insert(name, l.pop_front().ok_or("")?); ServValue::List(l) },
       	element => { scope.insert(name, element); ServValue::None },
   	};
   	
   	Ok(out)
}

fn with(mut input: VecDeque<ServValue>, scope: &mut Stack) -> ServResult {
    let result = crate::value::eval(input, scope)?.ignore_metadata();
    let ServValue::Table(m) = result else { panic!("take_keys expects a table, received {:?}", result) };

    for (k, v) in m.into_iter() {
        scope.insert(k.into(), v);
    }

    Ok(ServValue::None)
}

fn count(input: ServValue, scope: &Stack) -> ServResult {
    let max = input.expect_int()?;
    let mut output = VecDeque::new();
    let mut i: i64 = 0;

    while i < max {
        output.push_back(ServValue::Int(i));
		i += 1;
    }

	Ok(ServValue::List(output))
}


fn pop(input: ServValue, scope: &Stack) -> ServResult {
    let ServValue::List(mut inner) = input else { return Ok(ServValue::None) };
    _ = inner.pop_front();

    Ok(ServValue::List(inner))
}


fn get(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let output = match (arg.ignore_metadata(), input.ignore_metadata()) {
        (ServValue::Text(ref key), ServValue::Table(mut map)) => map.remove(key).ok_or("key not found")?,
        (ServValue::Int(index),    ServValue::List(mut list)) => list.remove(index.try_into().map_err(|e| "invalid index")?).ok_or("index not found")?,
        (_, _) => return Err("invalid key"),
    };

	Ok(output)
}

fn map(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let mut output = VecDeque::new();
    let ServValue::List(list) = input else { return arg.call(Some(input), scope) };
    for item in list.into_iter() {
        output.push_back(arg.call(Some(item), scope)?);
    }
    Ok(ServValue::List(output))
}

fn generate_list(input: VecDeque::<ServValue>, scope: &mut Stack) -> ServResult {
    let mut output = VecDeque::new();
    for item in input.into_iter() {
        output.push_back(item.call(None, scope)?);
    }
    Ok(ServValue::List(output))
}

fn table(input: ServValue, scope: &Stack) -> ServResult {
    let text = input.to_string();
    let ast = servparser::parse_root_from_text(&text).unwrap();
    let mut output = HashMap::new();
    let mut child = scope.make_child();

    for declaration in ast.0 {
        let value = crate::compile(declaration.value, &mut child).call(None, &child)?;
        if (declaration.kind == "include") {
            let text = value.to_string();
        	let ast = servparser::parse_root_from_text(&text).expect("include string failed to parse");
			crate::ast_bind_to_scope(ast, &mut child);
        } else {
            match declaration.key {
                crate::ast::Pattern::Expr(expr) => output.insert(crate::compile(expr, &mut child).call(None, &child)?.to_string(), value),
                crate::ast::Pattern::Key(name) => output.insert(name, value),
            };
        }
    }

    Ok(ServValue::Table(output))

}

fn sum(input: ServValue, scope: &Stack) -> ServResult {
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

pub fn bind(scope: &mut Stack) {
	scope.insert(Label::name("map"), ServValue::Func(ServFn::ArgFn(map)));

	scope.insert(Label::name("count"), ServValue::Func(ServFn::Core(count)));
	scope.insert(Label::name("|"),     ServValue::Func(ServFn::Meta(generate_list)));
	scope.insert(Label::name("pop"),   ServValue::Func(ServFn::Meta(take)));
	scope.insert(Label::name("<"),     ServValue::Func(ServFn::Meta(take)));
	scope.insert(Label::name("."),     ServValue::Func(ServFn::ArgFn(get)));
	scope.insert(Label::name("with"),     ServValue::Func(ServFn::Meta(with)));
	scope.insert(Label::name("sum"),     ServValue::Func(ServFn::Core(sum)));
	scope.insert(Label::name("@"),     ServValue::Func(ServFn::Core(table)));

}

// pub fn join(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
//     let intersperse = words.take_next(scope)?.to_string();
//     let ServValue::List(list) = words.eval(input, scope)?.ignore_metadata() else { return Err("sum needs to operate on a list") };
//     let t: Vec<String> = list.into_iter().map(|x| x.to_string()).collect();

//     Ok(ServValue::Text(t.join(&intersperse)))
// }


// pub fn quote(mut input: ServValue, scope: &Scope) -> ServResult {
//     Ok(input)
// }

// pub fn deref(input: ServValue, scope: &Scope) -> ServResult {
//     match input {
//         ServValue::List(i) => ServValue::Expr(i).eval(None, scope),
//         ServValue::Expr(i) => ServValue::Expr(i).eval(None, scope),
//         ServValue::ServFn(ref label) => scope.get(label).unwrap().call(ServValue::None, scope),
//         ref otherwise => Ok(input),
//     }
// }



// // pub fn map(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
// //     let next = words.next().ok_or("not enough arguments")?;
// //     let arg = scope.get(&next).ok_or("not found")?;
// //     let rest = words.eval(input, scope)?;

// // 	let mapped = match rest {
// //     	ServValue::List(list) => ServValue::List(list.into_iter().map(|a| arg.call(a, scope).unwrap()).collect()),
// //     	_ => todo!(),
// // 	};

// // 	Ok(mapped)
// // }

// pub fn fold(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
//     let next = words.next().ok_or("not enough arguments")?;
//     let arg = scope.get(&next).ok_or("not found")?;
//     let rest = words.eval(input, scope)?;

//     let mut acc = ServValue::None;

//     let ServValue::List(items) = rest else { return Err("not a list") };

//     for (index, item) in items.into_iter().enumerate() {
//         let mut child_scope = scope.make_child();
//         child_scope.insert_name("acc",   ServFunction::Literal(acc.clone()));
//         child_scope.insert_name("index", ServFunction::Literal(ServValue::Int(index.try_into().unwrap())));
//         acc = arg.call(item, &child_scope)?;
//     }

// 	Ok(acc)
// }


// pub fn split(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
//     let arg = words.take_next(scope)?.to_string();
//     let rest = words.eval(input, scope)?.to_string();

//     Ok(ServValue::List(rest.split(&arg).map(|x| ServValue::Text(x.to_owned())).collect()))
// }
