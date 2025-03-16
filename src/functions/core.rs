use crate::{ ServValue, ServResult, Label };
use crate::Stack;
use crate::servparser;
use crate::error::ServError;

use crate::ServModule;

use crate::value::{ ServFn, ServList };

fn serv_try(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
	let ServValue::Module(m) = arg else { return Err(ServError::expected_type("module", input)) };

	let mut child = scope.make_child();
	child.insert_module(m.values);

	let mut output: ServResult = Err(ServError::new(500, "empty try statement"));

    for mut expr in m.statements {
        output = expr.eval(&mut child);
        match &output {
			Err(_) => (),
			Ok(ServValue::None) => (),
			Ok(_) => return output,
        };
    }

	output
}

fn print(input: ServValue, scope: &Stack) -> ServResult {
    println!("{}", input);
    Ok(input)
}


fn uppercase(input: ServValue, scope: &Stack) -> ServResult {
    Ok(input.to_string().to_uppercase().into())
}

fn markdown(input: ServValue, scope: &Stack) -> ServResult {
    let compile_options = markdown::CompileOptions {
        allow_dangerous_html: true,
        allow_dangerous_protocol: true,
        gfm_tagfilter: false,
        ..markdown::CompileOptions::default()
    };

    let options = markdown::Options {
        compile: compile_options,
        ..markdown::Options::gfm()
    };

    let output = markdown::to_html_with_options(input.to_string().as_str(), &options).unwrap();
    Ok(ServValue::Text(output.into()))
}


fn drop(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    Ok(input)
}

fn using(mut input: ServList, scope: &mut Stack) -> ServResult {
    let mut arg = input.next().ok_or("using expects an arg")?;
    arg = arg.call(None, scope)?;

	let ServValue::Module(m) = arg else { return Err(ServError::expected_type("module", arg))};

	scope.insert_module(m.values);
	input.eval(scope)
}

fn as_template(input: ServValue, scope: &Stack) -> ServResult {
    let template = crate::servparser::parse_template_from_text(&input.to_string(), false).unwrap();
    template.render(&scope)
}

pub fn apply(input: ServValue, scope: &Stack) -> ServResult {
    let ServValue::Module(m) = input else { return Err(ServError::expected_type("Module", input)) };
    m.call(None, &mut scope.make_child())
}

fn with_headers(mut input: ServList, scope: &mut Stack) -> ServResult {
    let mut arg = input.pop()?;
	arg = arg.call(None, scope)?;

    let ServValue::Module(m) = &arg else {
        return Err(ServError::expected_type("Module", arg))
    };

    scope.insert("res.headers", arg);
    input.eval(scope)
}

fn dequote(input: ServValue, scope: &Stack) -> ServResult {
	if let ServValue::List(mut expr) = input {
    	expr.eval(&mut scope.make_child())
	}

	else {
    	Ok(input)
	}
}

fn quote(input: ServList, scope: &mut Stack) -> ServResult {
    Ok(ServValue::List(input))
}

fn choose(mut arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let ServValue::Module(m) = arg else {
        return Err(ServError::expected_type("module", arg));
    };

    let mut child = scope.make_child();
    let mut index = input.clone();

    if let Some(index_map) = m.values.get(&"i".into()) {
        index = index_map.call(Some(index), scope)?;
    }

    let mut path = match index {
        ServValue::None => m.statements.get(0).unwrap().clone(),
        ServValue::Bool(b) if b == true => m.statements.get(0).unwrap().clone(),
        ServValue::Bool(b)  => m.statements.get(1).unwrap().clone(),
        ServValue::Int(mut i) => {
            i = std::cmp::min(i, m.statements.len() as i64 - 1);
            i = std::cmp::max(i, 0);
            m.statements.get(i as usize).unwrap().clone()
        },

        other => panic!("not supported {:?}", other),
    };

    path.as_expr().call(Some(input), &mut child)

    // path.push_back(input);
    // path.eval(&mut child)
}

fn switch(mut arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
	arg = arg.call(None, scope)?;
    let ServValue::Module(m) = arg else { return Err(ServError::expected_type("Module", arg)) };
  //   for (p, a) in m.equalities {
  //       let pattern: ServValue = p.into();
  //       let action: ServValue = a.into();
		// if pattern.call(Some(input.clone()), scope).unwrap().is_truthy() {
  //   		return Ok(action.call(Some(input), scope).unwrap())
		// }
  //   }

    Ok(ServValue::None)
}


fn parse_module(input: ServValue, scope: &Stack) -> ServResult {
    let module = servparser::parse_root_from_text(&input.to_string()).unwrap();
    Ok(ServValue::Module(module))
}

pub fn assert(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let test = arg.call(Some(input.clone()), scope)?;

    if test.is_truthy() {
        return Ok(input)
    } else {
        return Err(ServError::new(500, "Assertion Failed"))
    }
}

pub fn get_module() -> ServModule {
    let mut output = ServModule::empty();
	output.insert("try",         ServFn::ArgFn(serv_try).into());
	output.insert("assert",      ServFn::ArgFn(assert).into());
	output.insert("print",       ServFn::Core(print).into());
	output.insert("[",           ServFn::Core(dequote).into());
	output.insert("]",           ServFn::Meta(quote).into());
	output.insert("using",       ServFn::Meta(using).into());
	output.insert("let",         ServFn::Meta(using).into());
	output.insert("!",           ServFn::ArgFn(drop).into());
	output.insert("?",           ServFn::ArgFn(choose).into());
	output.insert("switch",      ServFn::ArgFn(switch).into());
	output.insert("*",           ServFn::Core(apply).into());
	output.insert("uppercase",   ServFn::Core(uppercase).into());
	output.insert("markdown",    ServFn::Core(markdown).into());
	output.insert("with.header", ServFn::Meta(with_headers).into());
	output.insert("~",           ServFn::Core(as_template).into());
	output.insert("serv",        ServFn::Core(parse_module).into());
    output

}
