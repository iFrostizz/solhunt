// Can take all Modules to pass them to the walker

use crate::loader::DynModule;

pub struct Loader(pub Vec<DynModule>);

impl Loader
{
    pub fn new(modules: Vec<DynModule>) -> Self {
        Loader(modules)
    }
}
