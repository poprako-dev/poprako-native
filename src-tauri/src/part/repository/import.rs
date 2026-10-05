use poprako_orchestra::Oper;

use crate::data::import::{ExistingComicImport, NewComicImport};
use crate::model::comic::Comic;

#[derive(Oper)]
#[oper(output = Comic)]
pub struct CreateImportedComic {
    pub input: NewComicImport,
}

#[derive(Oper)]
#[oper(output = Vec<String>)]
pub struct ApplyComicImport {
    pub input: ExistingComicImport,
}
