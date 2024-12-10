use crate::ServValue;
use crate::ServResult;
use crate::Stack;
use crate::servparser;
use crate::VecDeque;
use crate::{Label, ServFn};


fn take(mut input: VecDeque<ServValue>, scope: &mut Stack) -> ServResult {
    let arg  = input.pop_front().expect("word expected");
   	let ServValue::Ref(name @ Label::Name(_)) = arg else { panic!("'<' expects a ref") };

   	let rest = crate::value::eval(input, scope)?;

   	let out = match rest {
       	ServValue::List(mut l) => { scope.insert(name, l.pop_front().ok_or("")?); ServValue::List(l) },
       	element => { scope.insert(name, element); ServValue::None },
   	};
   	
   	Ok(out)

   	// f(arg, rest, scope)

    // if let ServValue::Ref(Label::Name(ref name)) = arg {
    //     scope.insert("");
    //     let transform = Transform(Rc::new(|s: &mut Stack| { s.insert_name(name, ServValue::None) }));
    //     let out = ServValue::Transform(Box::new(input), transform);
    //     Ok(out)
    // }
    // else { return Ok(input) }

   	// todo!();
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


pub fn bind(scope: &mut Stack) {
	scope.insert(Label::name("map"), ServValue::Func(ServFn::ArgFn(map)));

	scope.insert(Label::name("count"), ServValue::Func(ServFn::Core(count)));
	scope.insert(Label::name("|"),     ServValue::Func(ServFn::Meta(generate_list)));
	scope.insert(Label::name("pop"),   ServValue::Func(ServFn::Meta(take)));
	scope.insert(Label::name("<"),     ServValue::Func(ServFn::Meta(take)));
	scope.insert(Label::name("."),     ServValue::Func(ServFn::ArgFn(get)));

}

// pub fn switch(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {

//     let arg1 = words.next().unwrap();
//     // let arg1 = (scope)?.expect_int()?;
//     let arg2 = words.next().ok_or("not enough arguments")?;
//     let ServFunction::List(options) = scope.get(&arg2).ok_or("not found")? else { return Err("switch needs its second arg to be a list literal") };

//     let rest = words.eval(input, scope)?;
//     let index: usize = scope.get(&arg1).unwrap().call(rest.clone(), scope)?.expect_int()?.try_into().unwrap();

// 	// let index: usize = arg1.try_into().unwrap();
//     let choice = options[index.clamp(0, options.len() - 1)].clone();

//     let output = scope.get(&choice).ok_or("not found")?.call(rest, scope)?;

// 	Ok(output)
// }

// pub fn sum(input: ServValue, scope: &Scope) -> ServResult {
//     let ServValue::List(list) = input.ignore_metadata() else { return Err("sum needs to operate on a list") };
//     let mut iter = list.into_iter().filter(|x| !matches!(x, ServValue::None)).peekable();

//     let output: ServValue = match iter.peek().ok_or("tried to sum an empty list")? {
//         ServValue::Int(i) => ServValue::Int(iter.map(|x| x.expect_int().unwrap()).sum()),
//         ServValue::Text(t) => {
//             let mut output = String::new();
//             for element in iter {
//                 output.push_str(element.to_string().as_str());
//             }
//             ServValue::Text(output)
//         },
//         _ => return Err("please don't try to sum that"),
//     };

// 	Ok(output)
// }

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
