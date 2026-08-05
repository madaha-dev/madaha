use std::fmt::{Debug, Display};

pub trait ConfigObject<E: Debug + Display>{
    fn check(&self) -> Result<(), E>;
    fn new() -> Self;
}
