pub mod dom;
pub mod error;
pub mod flow;
pub mod logging;
pub mod reactivity;

pub use error::{SinterError, SinterResult};

pub mod prelude {
    pub use crate::dom::*;
    pub use crate::error::{SinterError, SinterResult};
    pub use crate::flow::*;
    pub use crate::reactivity::{
        ReadSignal, Resource, RwSignal, WriteSignal, create_effect, create_memo, create_resource,
        create_rw_signal, create_scope, create_signal, on_cleanup, provide_context, use_context,
    };
}
