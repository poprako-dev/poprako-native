use std::path::PathBuf;

use poprako_orchestra::Oper;

#[derive(Clone, Copy)]
pub enum ImageSelectionKind {
    Files,
    Folder,
}

#[derive(Oper)]
#[oper(output = Option<Vec<PathBuf>>)]
pub struct SelectImages {
    pub kind: ImageSelectionKind,
}
