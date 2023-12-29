use std::collections::HashMap;

pub struct Table(HashMap<String, Value>);

impl Table {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let value = self.0.get(key)?;
        Some(value.to_string())
    }

    pub fn set<V>(&mut self, key: &str, value: V)
    where V: Into<Value>
    {
        self.0.insert(key.to_owned(), value.into());
    }
}

impl From<HashMap<String, String>> for Table {
    fn from(input: HashMap<String, String>) -> Self {
        let mut output: HashMap<String, Value> = HashMap::new();
        for (key, value) in input.into_iter() {
            output.insert(key, value.into());
        }

        return Self(output);
    }
}

pub enum Value {
	Empty,
	Text(String),
        Table(Table),
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Empty => String::new(),
            Value::Text(s) => s.to_string(),
            Value::Table(t) => todo!(),
        }
    }
}

impl From<&str> for Value {
    fn from(input: &str) -> Self {
        Self::Text(input.to_owned())
    }
}

impl From<String> for Value {
    fn from(input: String) -> Self {
        Self::Text(input)
    }
}

impl Default for &Value {
    fn default() -> Self {
        return &Value::Empty
    }
}

pub struct Response {
	pub headers: Vec<(String, String)>,
	pub body: Value
}

impl From<Response> for hyper::Response<hyper::Body> {
	fn from(input: Response) -> Self {
		let mut output = hyper::Response::builder().status(200);

		match input.body {
			Value::Text(t) => output.body(t.into()).unwrap(),
			_ => todo!(),
		}

		// output.body(value.body.into()).unwrap()
	}
}

