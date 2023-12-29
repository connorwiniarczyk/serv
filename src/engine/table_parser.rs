use super::Value;
use crate::parser::cursor::Cursor;
use crate::parser::MatchFail;
use std::collections::HashMap;

fn record(cursor: &mut Cursor) -> Result<(String, String), MatchFail> {
    cursor.skip(&[' ', '\t', '\n']); 
    let key = cursor.consume_identifier()?;
    cursor.skip(&[' ', '\t', '\n']); 
    cursor.expect("=")?;
    cursor.skip(&[' ', '\t', '\n']); 
    let value = cursor.consume_identifier()?;
    cursor.skip(&[' ', '\t', '\n']); 
    cursor.expect(";")?;
    Ok((key, value))
}

pub fn parse_table(input: &str) -> Value {
    let mut output: HashMap<String, Value> = HashMap::new();
	let text: Vec<char> = input.chars().collect();
	let mut cursor = Cursor::new(&text);

    while let Ok((key, value)) = record(&mut cursor) {
        output.insert(key.into(), value.into());
    }

    Value::Table(output)
}


