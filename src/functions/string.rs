/// functions for text

use crate::{ServValue, Stack, ServFn, ServModule, ServResult, ServString};

static HTMX_SRC: &[u8; 50917] = include_bytes!("htmx.min.js");

fn htmx_src(_input: ServValue, _scope: &Stack) -> ServResult {
    println!("{:?}", HTMX_SRC);
    Ok(ServString::from(HTMX_SRC.as_slice()).into())
}


/// Convert markdown text into HTML
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


/// Generate lorem ipsum dummy text
fn lorem(input: ServValue, scope: &Stack) -> ServResult {
    Ok(lipsum::lipsum(input.expect_int()?.try_into().unwrap()).into())
}

pub fn get_module() -> ServModule {
    let mut output = ServModule::empty();
	output.insert("markdown", ServFn::Core(markdown).into());
	output.insert("lorem", ServFn::Core(lorem).into());
	output.insert("jslib.htmx", ServFn::Core(htmx_src).into());
    output
}
