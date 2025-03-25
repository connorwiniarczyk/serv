use crate::ServValue;
use crate::ServResult;
use crate::Stack;
use crate::servparser;
use crate::{Label, ServFn};
use crate::ServError;
use crate::ServModule;
use crate::ServList;


use crate::template::{Template, TemplateElement, Renderer};
use crate::template;

use std::collections::VecDeque;
use std::collections::HashMap;

use std::sync::Arc;

use crate::dictionary::DatabaseConnection;

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


// fn get_database_location(scope: &Stack) -> Result<String, ServError> {
//     // let value = scope.get("sqlite.database")?.call(None, scope)?;
//     let value = crate::engine::deref(&"sqlite.database".into(), scope)?.call(None, scope)?;

//     if let ServValue::Text(output) = value {
//         Ok(output.as_str()?.to_string())
//     } else {
//     	Err(ServError::expected_type("string", value))
//     }
// }

fn sqlite_exec(input: ServValue, scope: &Stack) -> ServResult {
    let DatabaseConnection::Sqlite(connection) = scope.get_database_connection().expect("not connected to a database");
    connection.execute(input.to_string()).unwrap();
    Ok(ServValue::None)
}

fn sqlite_bind_param(statement: &mut sqlite::Statement, i: usize, param: ServValue, scope: &Stack) -> Result<(), ServError> {
    match param {
        ServValue::Ref(ref addr) => sqlite_bind_param(statement, i, crate::engine::deref(addr, scope)?, scope)?,
        ServValue::Func(_)   => {
            let result = engine::resolve(param, None, scope)?;
            sqlite_bind_param(statement, i, result, scope)?
        },
        ServValue::Int(v)    => statement.bind((i, v)).unwrap(),
        ServValue::Text(t)   => statement.bind((i, t.as_str()?)).unwrap(),
        ServValue::Float(v)  => statement.bind((i, v)).unwrap(),
        ServValue::None      => statement.bind((i, ())).unwrap(),
        ServValue::Bool(v)   => todo!(),
        ServValue::List(v)   => todo!(),
        ServValue::Table(v)  => todo!(),

        ref otherwise => panic!("{:?}", param),
    };

    Ok(())
}

fn sqlite_query(mut arg: ServValue, input: ServValue, s: &Stack) -> ServResult {
    let DatabaseConnection::Sqlite(connection) = s.get_database_connection().unwrap();

    let mut child = s.make_child();
    child.insert("in", input.clone());
    child.insert("x", input);
    let scope = &child;

	while let ServValue::Ref(addr) = arg {
    	arg = crate::engine::deref(&addr, scope)?;
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
                sqlite::Type::Null    => ServValue::None,
                sqlite::Type::String  => statement.read::<String, _>(index).unwrap().into(),

            };
			row.insert(name.clone(), value);
        }
        output.push(ServValue::Table(row));
    }

    Ok(ServValue::List(output.into()))
}

use crate::engine;

fn sqlite_connect(mut input: ServList, ctx: &mut Stack) -> ServResult {
    let location = engine::eval(input, ctx)?.to_string();
    let connection = sqlite::Connection::open_thread_safe(location.as_str()).unwrap();

    ctx.connection = Some(DatabaseConnection::Sqlite(connection));
    Ok(ServValue::None)

 //    let mut output = ServModule::empty();
	// output.insert("query", ServFn::ArgFn(sqlite_query).into());
	// output.insert("run",   ServFn::Core(sqlite_exec).into());
	// Ok(output.into())
}

pub fn get_module() -> ServModule {
    let mut output = ServModule::empty();
	output.insert("sqlite.connect", ServFn::Meta(sqlite_connect).into());
	output.insert("sqlite.query",   ServFn::ArgFn(sqlite_query).into());
	output.insert("sqlite.run",     ServFn::Core(sqlite_exec).into());
	output
}
