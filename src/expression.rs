

enum Value {
    Text(String), Table, Stream
}

enum ExpressionInterpreter {
    PlainText,
    Html,
    ReadFile,
}

enum AstToken {
    Text(String),
    Variable(String),
    
    Expression {
        interpreter: ExpressionInterpreter,
        content: Vec<AstToken>,
    },
}

impl AstToken {
    pub fn into_value(&self, engine: &mut Engine) -> Value {
        match self {
            Self::Text(v) => Value::Text(v.clone()),
            Self::Variable(v) => engine.get_var(&v).unwrap(),
            Self::Expression { interpreter, content } => {
                match interpreter {
                    ExpressionInterpreter::PlainText => {
                        let mut output = String::new();
                        for t in content {
                            let new_text = match t.into_value(engine) {
                                Value::Text(t) => t,
                                _ => todo!(),
                            };
                            output.push_str(&new_text);
                        }
                        Value::Text(output)
                    },
                    _ => todo!(),
                }
            },
            _ => todo!(),
        }
    }
}

struct Engine;

impl Engine {
    fn get_var(&self, path: &str) -> Option<Value> {
        todo!();
    }
}
