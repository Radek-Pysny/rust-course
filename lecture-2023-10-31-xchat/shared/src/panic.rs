use std::any::Any;


/// `panic_to_text` help with conversion of panic error message; especially while joining thread
/// handles.
pub fn panic_to_text(error: Box<dyn Any + Send>) -> String {
    if let Some(str) = error.downcast_ref::<String>() {
        return str.clone();
    }

    if let Ok(str) = error.downcast::<&str>() {
        return str.to_string();
    }

    return "Unknown error".to_string();
}
