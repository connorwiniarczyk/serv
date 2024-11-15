use crate::{Scope, ServValue, ServResult, FnLabel, ServFunction, Words};
use std::collections::HashMap;

fn get_database_location(scope: &Scope) -> Option<String> {
	let output = scope.get_str("sql.database").ok()?.call(ServValue::None, scope).ok()?.to_string();
	Some(output)
}

// fn sql_get_query(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
//     let arg = words.next().ok_or("not enough arguments")?;
//     let ServFunction::Template(template) = scope.get(&arg).ok_or("not found")? else { return Err("not a template") };

//     let (output, sql_bindings) = template.render_sql(scope);

//     todo!();
// }

fn sql_exec(input: ServValue, scope: &Scope) -> ServResult {
    let path = get_database_location(scope).unwrap_or("serv.sqlite".to_string());
    let connection = sqlite::open(&path).unwrap();
    connection.execute(input.to_string()).unwrap();
    Ok(ServValue::None)
}

fn sql(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let path = get_database_location(scope).unwrap_or("serv.sqlite".to_string());
    let connection = sqlite::open(&path).unwrap();

	let mut arg = scope.get(&words.next().ok_or("")?).ok_or("")?;
	let ServFunction::Template(t) = arg else {panic!()};
	let (ctx, query, params) = t.render_sql(scope);
    let mut statement = connection.prepare(query.to_string()).unwrap();

    let rest = words.eval(input, scope)?;

    for (mut i, p) in params.into_iter().enumerate() {
        i += 1;
        let value = ctx.get(&p).ok_or("function not found")?.call(rest.clone(), &ctx).unwrap();
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

// fn sql(input: ServValue, scope: &Scope) -> ServResult {
//     let mut output: Vec<ServValue> = Vec::new();
//     let path = get_database_location(scope).unwrap_or("serv.sqlite".to_string());
//     let connection = sqlite::open(&path).unwrap();

//     let mut statement = connection.prepare(input.to_string()).unwrap();

//     while let Ok(sqlite::State::Row) = statement.next() {
//         let mut row: HashMap<String, ServValue> = HashMap::new();
//         for (index, name) in statement.column_names().iter().enumerate() {
//             let value = match statement.column_type(index).unwrap() {
//                 sqlite::Type::Binary  => {
//                     let v: i64 = statement.read(index).unwrap();
//                     ServValue::Bool(if v == 0 {false} else {true})
//                 },
//                 sqlite::Type::Float   => ServValue::Float(statement.read(index).unwrap()),
//                 sqlite::Type::Integer => ServValue::Int(statement.read(index).unwrap()),
//                 sqlite::Type::String  => ServValue::Text(statement.read(index).unwrap()),
//                 sqlite::Type::Null    => ServValue::None,

//             };
// 			row.insert(name.clone(), value);
//         }
//         output.push(ServValue::Table(row));
//     }

//     Ok(ServValue::List(output.into()))
// }

pub fn bind(scope: &mut Scope) {
	scope.insert(FnLabel::name("sql"),    ServFunction::Meta(sql));
	scope.insert(FnLabel::name("sql.exec"),    ServFunction::Core(sql_exec));
}
