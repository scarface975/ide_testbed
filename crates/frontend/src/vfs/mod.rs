use std::rc::Rc;

use futures_signals::{signal::Mutable, signal_vec::MutableVec};

#[derive(Clone)]
pub struct File {
    pub name: Mutable<String>,
    pub mode: Mutable<u32>,
    pub data: Mutable<Vec<u8>>,
}

#[derive(Clone)]
pub struct Directory {
    pub name: Mutable<String>,
    pub mode: Mutable<u32>,
    pub directories: MutableVec<Rc<Directory>>,
    pub files: MutableVec<Rc<File>>
}

