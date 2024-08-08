use crate::{Scope, ServValue, ServResult, FnLabel, ServFunction};

use std::collections::HashMap;

struct QueryCursor<'input> {
    input: &'input [char],
    mark: usize,
    index: usize,
}

impl<'input> QueryCursor<'input> {
    fn new(input: &'input [char]) -> Self {
        Self { input, mark: 0, index: 0}
    }

    fn emit(&mut self) -> String {
        let mut output = String::new();
        for i in self.mark..self.index {
            output.push(self.input[i]);
        }
        self.mark = self.index;
        println!("{}", output);
        output
    }

    fn incr_while<F>(&mut self, test: F) where F: Fn(char) -> bool {
        while (self.index < self.input.len() && (test)(self.input[self.index])) {
            self.index += 1;
        }
    }

    fn is_done(&self) -> bool {
		self.mark >= self.input.len()
    }

    fn skip(&mut self) {
        if self.index < self.input.len() {
            self.index += 1;
        }

        self.mark = self.index;
    }
}

fn parse_query_string(input: &str) -> ServValue {
    let mut output: HashMap<String, ServValue> = HashMap::new();

    let chars: Vec<char> = input.chars().collect();
    let mut cursor = QueryCursor::new(&chars);

    while !cursor.is_done() {
        cursor.incr_while(|c| c != '=');
        let key = cursor.emit();
        cursor.skip();
        cursor.incr_while(|c| c != '&');
        let value = cursor.emit();
        cursor.skip();
        output.insert(key, ServValue::Text(value));
    }

    ServValue::Table(output)
}

fn query_all(input: ServValue, scope: &Scope) -> ServResult {
    let Some(req) = scope.get_request() else { return Ok(ServValue::None) };
    let Some(query) = req.uri.query() else { return Ok(ServValue::None) };
    let table = parse_query_string(&query);
    Ok(table)
}

fn query_get(input: ServValue, scope: &Scope) -> ServResult {
    todo!();
}

pub fn bind(scope: &mut Scope) {
	scope.insert(FnLabel::name("query"), ServFunction::Core(query_all));
}
