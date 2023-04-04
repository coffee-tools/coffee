//! Core macros implemented for coffee.
#[macro_export]
macro_rules! error {
    ($($msg:tt)*) => {{
        let msg = format!($($msg)*);
        CoffeeError::new(1, &msg)
    }};
}

pub use error;
