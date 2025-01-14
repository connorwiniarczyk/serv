use crate::ServValue;
use crate::ServResult;
use crate::ServError;
use crate::Stack;
use crate::servparser;
use crate::{Label, ServFn};
use crate::value::ServList;

use std::collections::VecDeque;
use std::collections::HashMap;

fn take(mut input: ServList, scope: &mut Stack) -> ServResult {
    let arg = input.pop()?;
   	let ServValue::Ref(name @ Label::Name(_)) = arg else { panic!("'<' expects a ref") };
   	let rest = input.eval(scope)?;
   	let out = match rest {
       	ServValue::List(mut l) => {
           	scope.insert(name, l.pop()?);
           	ServValue::List(l)
       	},
       	element => {
           	scope.insert(name, element);
           	ServValue::None
       	},
   	};
   
   	Ok(out)
}

fn with(mut expr: ServList, scope: &mut Stack) -> ServResult {
    let input = expr.eval(scope)?;
    match input {
        ServValue::Table(t)  => t.into_iter().for_each(|(ref key, value)| scope.insert_name(key, value)),
        ServValue::Module(t) => {
            for (key, mut expr) in t.definitions.into_iter() {
                let value = expr.eval(scope)?;
                scope.insert(key, value);
            }
        },
        otherwise => return Err(ServError::expected_type("Table | Module", otherwise)),
    };

    Ok(ServValue::None)
}

fn count(input: ServValue, scope: &Stack) -> ServResult {
    let max = input.expect_int()?;
    let mut output = ServList::new();
    let mut i: i64 = 0;

    while i < max {
        output.push_back(ServValue::Int(i));
		i += 1;
    }

	Ok(ServValue::List(output))
}


fn pop(input: ServValue, scope: &Stack) -> ServResult {
    todo!();
    // if let ServValue::List(mut list) = input {
    //     list.pop()
    // } else {

    // }
    // let ServValue::List(mut inner) = input else { return Ok(ServValue::None) };
    // _ = inner.pop_front();
    // Ok(ServValue::List(inner))
}


fn get(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let output = match (arg, input) {
        (ServValue::Text(ref key), ServValue::Table(mut map)) => map.remove(key.as_str()?).ok_or("key not found")?,
        (ServValue::Int(index),    ServValue::List(mut list)) => list.get(index.try_into().map_err(|e| "invalid index")?)?.clone(),
        (key, _) => return Err(ServError::expected_type("Int | Text", key)),
    };

	Ok(output)
}

fn map(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let mut output = ServList::new();
    let ServValue::List(list) = input else { return arg.call(Some(input), scope) };
    for item in list {
        output.push_back(arg.call(Some(item), scope)?);
    }
    Ok(ServValue::List(output))
}

fn generate_list(input: ServList, scope: &mut Stack) -> ServResult {
    let mut output = ServList::new();

    for item in input {
        output.push_back(item.call(None, scope)?);
    }
    Ok(ServValue::List(output))
}

fn sum(mut input: ServValue, scope: &Stack) -> ServResult {
    let ServValue::List(list) = input else {
        return Err(ServError::expected_type("List", input))
    };

    let mut iter = list.filter(|x| !matches!(x, ServValue::None)).peekable();

    let output: ServValue = match iter.peek().ok_or("tried to sum an empty list")? {
        ServValue::Int(i) => ServValue::Int(iter.map(|x| x.expect_int().unwrap()).sum()),
        ServValue::Text(t) => {
            let mut output = String::new();
            for element in iter {
                output.push_str(element.to_string().as_str());
            }
            output.into()
        },
        otherwise => return Err(ServError::expected_type("Int | Text", otherwise.clone())),
    };

	Ok(output)
}

pub fn bind(scope: &mut Stack) {
	scope.insert(Label::name("map"), ServValue::Func(ServFn::ArgFn(map)));

	scope.insert(Label::name("count"), ServValue::Func(ServFn::Core(count)));
	scope.insert(Label::name("|"),     ServValue::Func(ServFn::Meta(generate_list)));
	scope.insert(Label::name("pop"),   ServValue::Func(ServFn::Meta(take)));
	scope.insert(Label::name("<"),     ServValue::Func(ServFn::Meta(take)));
	scope.insert(Label::name(":"),     ServValue::Func(ServFn::ArgFn(get)));
	scope.insert(Label::name("with"),  ServValue::Func(ServFn::Meta(with)));
	scope.insert(Label::name("sum"),   ServValue::Func(ServFn::Core(sum)));

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
