use crate::{Scope, ServValue, ServResult, FnLabel, ServFunction, Words};
use std::collections::HashMap;

fn get_database_location(scope: &Scope) -> Option<String> {
	let output = scope.get_str("sql.database").ok()?.call(ServValue::None, scope).ok()?.to_string();
	Some(output)
}

fn sql_get_query(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {
    let arg = words.next().ok_or("not enough arguments")?;
    let ServFunction::Template(template) = scope.get(&arg).ok_or("not found")? else { return Err("not a template") };

    let (output, sql_bindings) = template.render_sql(scope);



    todo!();
}

fn sql_exec(input: ServValue, scope: &Scope) -> ServResult {
    let path = get_database_location(scope).unwrap_or("serv.sqlite".to_string());
    let connection = sqlite::open(&path).unwrap();
    connection.execute(input.to_string()).unwrap();
    Ok(ServValue::None)
}

// pub fn get(words: &mut Words, input: ServValue, scope: &Scope) -> ServResult {

fn sql(words: Words, input: ServValue, scope: &Scope) -> ServResult {
    let mut output: Vec<ServValue> = Vec::new();
    let path = get_database_location(scope).unwrap_or("serv.sqlite".to_string());
    let connection = sqlite::open(&path).unwrap();

    let mut statement = connection.prepare(input.to_string()).unwrap();

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
