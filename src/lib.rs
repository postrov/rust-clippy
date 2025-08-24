mod common;
mod decode;
mod delete;
mod error;
mod list;
mod store;

pub use decode::*;
pub use delete::*;
pub use error::{Error, Result};
pub use list::*;
pub use store::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        let result = 4;
        assert_eq!(result, 4);
    }
}
