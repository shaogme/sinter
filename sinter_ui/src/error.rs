use std::fmt;

#[derive(Debug, Clone)] // Clone to allow easy propagation in closures if needed
pub enum SinterError {
    Dom(String),
    Reactivity(String),
    Javascript(String),
}

#[derive(Clone)]
pub struct ErrorContext(pub std::rc::Rc<dyn Fn(SinterError)>);

impl fmt::Display for SinterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SinterError::Dom(msg) => write!(f, "DOM Error: {}", msg),
            SinterError::Reactivity(msg) => write!(f, "Reactivity Error: {}", msg),
            SinterError::Javascript(msg) => write!(f, "JavaScript Error: {}", msg),
        }
    }
}

impl std::error::Error for SinterError {}

impl From<wasm_bindgen::JsValue> for SinterError {
    fn from(value: wasm_bindgen::JsValue) -> Self {
        let msg = value.as_string().unwrap_or_else(|| format!("{:?}", value));
        SinterError::Javascript(msg)
    }
}

pub type SinterResult<T> = Result<T, SinterError>;

pub fn handle_error(err: SinterError) {
    if let Some(ctx) = crate::reactivity::use_context::<ErrorContext>() {
        (ctx.0)(err);
    } else {
        crate::error!("Unhandled Sinter Error: {:?}", err);
    }
}
