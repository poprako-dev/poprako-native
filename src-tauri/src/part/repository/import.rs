use poprako_orchestra::Oper;

use crate::data::import::{ExistingChapterImport, NewChapterImport};
use crate::model::chapter::Chapter;

#[derive(Oper)]
#[oper(output = Chapter)]
pub struct CreateImportedChapter {
    pub input: NewChapterImport,
}

#[derive(Oper)]
#[oper(output = Vec<String>)]
pub struct ApplyChapterImport {
    pub input: ExistingChapterImport,
}
