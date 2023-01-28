use super::Cmd;
use async_trait::async_trait;
use crate::request_state::RequestState;
use rhai::{Engine, EvalAltResult};

pub struct Script {
    value: String,
}

#[derive(Clone)]
struct Test {
    x: i32,
    y: i32,
}

impl Test {
    fn swap(&mut self) {
        let temp = self.x;
        self.x = self.y;
        self.y = temp;
    }

    fn new() -> Self {
        Self { x: 100, y: 200 }
    }
}


#[async_trait]
impl Cmd for Script {
    fn name(&self) -> &str { "rhai" }
    fn arg(&self) -> &str { "" }

    fn with_arg(arg: Option<&str>) -> Self where Self: Sized {
        Self { value: arg.unwrap_or("").to_owned() }
    }

    async fn run(&self, state: &mut RequestState) {
        let mut engine = Engine::new();
        engine.register_type::<Test>();
        engine.register_fn("new_test", Test::new);
        engine.register_fn("swap", Test::swap);
        engine.register_get("x",  );

        engine.run(self.value.as_str()).unwrap();
    }
}
