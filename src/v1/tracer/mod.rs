#[derive(Debug, Clone)]
pub struct Tracer {
  pub etable: String,
}

impl Tracer {
  pub fn new() -> Self {
    Tracer {
      etable: String::new(),
    }
  }
}