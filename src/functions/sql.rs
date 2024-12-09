use crate::ServValue;
use crate::ServResult;
use crate::Stack;
use crate::servparser;
use crate::VecDeque;
use crate::{Label, ServFn};

use std::collections::HashMap;

fn get_database_location(scope: &Stack) -> Option<String> {
	let func = scope.get("sql.database")?;
	Some(func.call(None, scope).ok()?.to_string())
}

fn sql_exec(input: ServValue, scope: &Stack) -> ServResult {
    let path = get_database_location(scope).unwrap_or("serv.sqlite".to_string());
    let connection = sqlite::open(&path).unwrap();
    connection.execute(input.to_string()).unwrap();
    Ok(ServValue::None)
}

fn sql(arg: ServValue, input: ServValue, scope: &Stack) -> ServResult {
    let path = get_database_location(scope).unwrap_or("serv.sqlite".to_string());
    let connection = sqlite::open(&path).unwrap();

	let ServValue::Ref(r) = arg else {panic!()};
	let ServValue::Func(ServFn::Template(t)) = scope.get(r).unwrap() else {panic!()};

	let (ctx, query, params) = t.render_sql(scope);
    let mut statement = connection.prepare(query.to_string()).unwrap();

    for (mut i, p) in params.into_iter().enumerate() {
        i += 1;
        let value = scope.get(p).ok_or("function not found")?.call(Some(input.clone()), &scope).unwrap();
        match value {
            ServValue::Int(v)    => statement.bind((i, v)),
            ServValue::Text(t)   => statement.bind((i, t.as_str())),
            ServValue::Float(v)  => statement.bind((i, v)),
            ServValue::None      => statement.bind((i, ())),
            ServValue::Bool(v)   => todo!(),
            ServValue::Raw(v)    => todo!(),
            ServValue::List(v)   => todo!(),
            ServValue::Table(v)  => todo!(),
            ServValue::Meta {..} => todo!(),

            otherwise => todo!(),
        }.expect("failed to bind to sqlite statement");
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
