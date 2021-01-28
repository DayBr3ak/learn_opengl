mod bindings;
use bindings::Gles2 as InnerGl;
pub use bindings::*;

use std::{ops::Deref, rc::Rc};

#[derive(Clone)]
pub struct Gles {
    inner: Rc<InnerGl>,
}

impl Gles {
    pub fn load_with<F>(loadfn: F) -> Gles
    where
        F: FnMut(&'static str) -> *const types::GLvoid,
    {
        Gles {
            inner: Rc::new(InnerGl::load_with(loadfn)),
        }
    }
}

impl Deref for Gles {
    type Target = InnerGl;

    fn deref(&self) -> &InnerGl {
        &self.inner
    }
}
