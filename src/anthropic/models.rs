use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct Model(String);

pub static SONNET: LazyLock<Model> = LazyLock::new(|| Model::from("claude-3-5-sonnet-latest"));
pub static HAIKU: LazyLock<Model> = LazyLock::new(|| Model::from("claude-3-5-haiku-latest"));

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl From<&str> for Model {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl Model {}
