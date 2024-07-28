use crate::Scope;
use crate::FnLabel;
use crate::VecDeque;
use crate::ServValue;
use crate::ServResult;
use crate::ServFunction;

pub struct Words(pub VecDeque<FnLabel>);

impl Words {
    pub fn next(&mut self) -> Option<FnLabel> {
        self.0.pop_front()
    }

    pub fn empty() -> Self {
        Self(VecDeque::new())
    }

    pub fn take_next(&mut self, scope: &Scope) -> ServResult {
        let next_word = self.next().ok_or("not enough arguments")?;
        let func = scope.get(&next_word).ok_or("word not found")?;
        let output = func.call(ServValue::None, scope)?;

		Ok(output)
    }

    pub fn eval(&mut self, input: ServValue, scope: &Scope) -> ServResult {
        let Some(next) = self.next() else { return Ok(input) };
        let Some(next_fn) = scope.get(&next) else { panic!("word not found: {:?}", next)};

        if let ServFunction::Meta(m) = next_fn {
			return (m)(self, input, scope);
        } else {
            let rest = self.eval(input, scope)?;
            return next_fn.call(rest, scope)
        }
    }
}
