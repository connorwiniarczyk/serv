use crate::ServValue;
use crate::ServType;
use crate::ServResult;
use crate::ServError;
use crate::Stack;

use crate::{Label, ServFn};
use crate::value::ServList;
use crate::ServModule;

// use crate::servparser;

use crate::engine;

use std::collections::VecDeque;
use std::collections::HashMap;

fn list(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let m = arg.expect_module()?;

    let mut output = ServList::new();
    let mut child = scope.make_child();
    child.insert_module(m.values);

    for s in m.statements {
		output.push_back(s.as_expr().call(Some(input.clone()), &child)?);
    }

    Ok(output.into())
}

fn take(mut input: ServList, scope: &mut Stack) -> ServResult {
    let addr = input.pop()?.expect_ref()?;

   	// let ServValue::Ref(addr) = arg else { panic!("'<' expects a ref") };

   	let rest = input.eval(scope)?;
   	let out = match rest {
       	ServValue::List(mut l) => {
           	scope.insert(addr, l.pop()?);
           	ServValue::List(l)
       	},
       	element => {
           	scope.insert(addr, element);
           	ServValue::None
       	},
   	};
   
   	Ok(out)
}

fn with(mut expr: ServList, scope: &mut Stack) -> ServResult {
    let input = expr.eval(scope)?;
    match input {
        ServValue::Table(t)  => {
            for (key, value) in t.into_iter() {
                scope.insert(key.as_str(), value)?;
            }
        }
        ServValue::Module(t) => scope.insert_module(t.values),
        otherwise => return Err(ServError::expected_type(ServType::Module, otherwise)),
    };

    Ok(ServValue::None)
}

fn using(mut expr: ServList, scope: &mut Stack) -> ServResult {
    let arg = engine::resolve(expr.pop()?, None, scope)?;

    match arg {
        ServValue::Table(t)  => {
            for (key, value) in t.into_iter() {
                scope.insert(key.as_str(), value)?;
            }
        }
        ServValue::Module(t) => scope.insert_module(t.values),
        otherwise => return Err(ServError::expected_type(ServType::Module, otherwise)),
    };

	expr.eval(scope)

    // Ok(ServValue::None)
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

        (key, _) => return Err(ServError::expected_type(ServType::Text, key)),
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
        return Err(ServError::expected_type(ServType::List, input))
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
        otherwise => return Err(ServError::expected_type(ServType::Int, otherwise.clone())),
    };

	Ok(output)
}

pub fn get_module() -> ServModule {
    let mut output = ServModule::empty();

	output.insert("map",   ServFn::ArgFn(map).into());
	output.insert("list",  ServFn::ArgFn(list).into());
	output.insert("count", ServFn::Core(count).into());
	output.insert("|",     ServFn::Meta(generate_list).into());
	output.insert("pop",   ServFn::Meta(take).into());
	output.insert("<",     ServFn::Meta(take).into());
	output.insert(":",     ServFn::ArgFn(get).into());
	output.insert("with",  ServFn::Meta(with).into());
	output.insert("using", ServFn::Meta(using).into());
	output.insert("let",   ServFn::Meta(using).into());
	output.insert("sum",   ServFn::Core(sum).into());

	output
}
