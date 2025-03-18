use std::iter::Peekable;

use crate::{ServValue, ServError, Stack, Address, Label, ServFn};

pub fn resolve(func: ServValue, input: Option<ServValue>, scope: &Stack) -> Result<ServValue, ServError> {
   	match func {
       	ServValue::Ref(ref addr) => resolve(deref(addr, scope)?, input, scope),
		ServValue::Module(m) => m.clone().call(input, &mut scope.make_child()),

       	ServValue::Func(ServFn::Core(f)) => f(input.unwrap_or_default(), scope),
       	ServValue::Func(ServFn::Expr(e, _)) => {
           	let mut child = scope.make_child();
           	let mut expr = e.clone();
           	if let Some(v) = input { expr.push_back(v) };
           	expr.eval(&mut child)
       	},

       	ServValue::Func(ServFn::Template(t)) => {
           	if let Some(v) = input {
               	let mut child = scope.make_child();
               	child.insert("in", v.clone());
               	child.insert("x", v);
               	t.render(&child)
           	}

           	else {
               	t.render(scope)
           	}
      	},

       	constant => Ok(constant.clone()),
   	}
}

// fn deref_internal<'a, 'b, I: Iterator<Item = &'a Label>>(mut value: &'b ServValue, q: &mut Peekable<I>, scope: &'b Stack) -> Result<&'b ServValue, ServError> {
fn deref_internal<'a, I: Iterator<Item = &'a Label>>(mut value: ServValue, q: &mut Peekable<I>, scope: &Stack) -> Result<ServValue, ServError> {
    // println!("{:?}", value);
    match value {
        ServValue::Ref(ref addr) => deref_internal(deref(addr, scope)?, q, scope),
        ServValue::Func(_) if q.peek().is_some() => deref_internal(resolve(value, None, scope)?, q, scope),

        ServValue::Module(ref m) => {
            // println!("{:?}", m);
            let Some(next) = q.next() else {
				return Ok(value)
            };

            let child = m.values.get(next).unwrap();
            deref_internal(child.clone(), q, scope)
        },

        ref otherwise => {
            let Some(next) = q.next() else {
				return Ok(value)
            };

            return Err(ServError::new(500, "not found"))
        },
    }
}

pub fn deref(addr: &Address, scope: &Stack) -> Result<ServValue, ServError> {
    let mut iter = addr.iter().peekable();
    let key = iter.next().ok_or(ServError::new(500, "empty address"))?;
    let value = scope.get(key.clone())?;

    return deref_internal(value, &mut iter, scope)
}
