use crate::ServValue;
use crate::ServResult;
use crate::Stack;
use crate::servparser;
use crate::{Label, ServFn};
use crate::ServError;

use crate::template::{Template, TemplateElement, Renderer};
use crate::template;

use std::collections::VecDeque;
use std::collections::HashMap;

struct SqliteRenderer(Vec<ServValue>);

type Buffer<'a> = &'a mut (dyn std::fmt::Write + 'a);

impl template::Renderer for SqliteRenderer {
    fn render<'buf>(&mut self, input: &template::Template, dest: Buffer<'buf>) {
        for element in &input.elements {
            match element {
                TemplateElement::Text(t) => dest.write_str(t),
                TemplateElement::Expression(e) => {
                    self.0.push(e.clone());
                    dest.write_str("?")
                },
                TemplateElement::Template(inner) => {
					dest.write_str(&inner.open);
					self.render(inner, dest);
					dest.write_str(&inner.close)
                },
            };
        }
    }
}


fn get_database_location(scope: &Stack) -> Option<String> {
	let func = scope.get("sql.database").ok()?;
	Some(func.call(None, scope).ok()?.to_string())
}

fn sql_exec(input: ServValue, scope: &Stack) -> ServResult {
    let path = get_database_location(scope).unwrap_or("serv.sqlite".to_string());
    let connection = sqlite::open(&path).unwrap();
    connection.execute(input.to_string()).unwrap();
    Ok(ServValue::None)
}

fn sqlite_bind_param(statement: &mut sqlite::Statement, i: usize, param: ServValue, scope: &Stack) -> Result<(), ServError> {
    match param {
        ServValue::Ref(label) => {
            let value = scope.get(label)?;
            sqlite_bind_param(statement, i, value, scope);
        },
        ServValue::Int(v)    => statement.bind((i, v)).unwrap(),
        ServValue::Text(t)   => statement.bind((i, t.as_str())).unwrap(),
        ServValue::Float(v)  => statement.bind((i, v)).unwrap(),
        ServValue::None      => statement.bind((i, ())).unwrap(),
        ServValue::Bool(v)   => todo!(),
        ServValue::Raw(v)    => todo!(),
        ServValue::List(v)   => todo!(),
        ServValue::Table(v)  => todo!(),

        otherwise => todo!(),
    };

    Ok(())
}

fn sql(mut arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let path = get_database_location(scope).unwrap_or("serv.sqlite".to_string());
    let connection = sqlite::open(&path).unwrap();

	if let ServValue::Ref(label) = arg {
    	arg = scope.get(label).unwrap();
	}

	let ServValue::Func(ServFn::Template(t)) = arg else {panic!()};

    let mut r = SqliteRenderer(Vec::new());
    let mut query = String::new();
    r.render(&t, &mut query);

    let mut statement = connection.prepare(query).unwrap();
    for (mut i, p) in r.0.into_iter().enumerate() {
        i += 1;
        sqlite_bind_param(&mut statement, i, p, scope).unwrap();
    }


    let mut output: Vec<ServValue> = Vec::new();
    while let Ok(sqlite::State::Row) = statement.next() {
        let mut row: HashMap<String, ServValue> = HashMap::new();
        for (index, name) in statement.column_names().iter().enumerate() {
            let value = match statement.column_type(index).unwrap() {
                sqlite::Type::Binary  => {
                    let v: i64 = statement.read(index).unwrap();
                    ServValue::Bool(if v == 0 {false} else {true})
                },
                sqlite::Type::Float   => ServValue::Float(statement.read(index).unwrap()),
                sqlite::Type::Integer => ServValue::Int(statement.read(index).unwrap()),
                sqlite::Type::String  => ServValue::Text(statement.read(index).unwrap()),
                sqlite::Type::Null    => ServValue::None,

            };
			row.insert(name.clone(), value);
        }
        output.push(ServValue::Table(row));
    }

    Ok(ServValue::List(output.into()))
}

pub fn bind(scope: &mut Stack) {
	scope.insert(Label::name("sql"), ServValue::Func(ServFn::ArgFn(sql)));
	scope.insert(Label::name("sql.exec"),  ServValue::Func(ServFn::Core(sql_exec)));
}
