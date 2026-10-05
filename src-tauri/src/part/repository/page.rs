use poprako_orchestra::Oper;

use crate::data::page::{PageBaseline, RemovePages, ReorderPages};
use crate::model::page::Page;
use crate::value::image::Image;

#[derive(Oper)]
#[oper(output = Vec<Page>)]
pub struct AppendPages {
    pub comic_id: String,
    pub baseline: Vec<PageBaseline>,
    pub pages: Vec<crate::data::page::NewPage>,
}

#[derive(Oper)]
#[oper(output = Vec<Page>)]
pub struct PersistPageOrder {
    pub input: ReorderPages,
}

#[derive(Oper)]
#[oper(output = Vec<Page>)]
pub struct DeletePages {
    pub input: RemovePages,
}

#[derive(Oper)]
#[oper(output = Page)]
pub struct ReplacePageImage {
    pub comic_id: String,
    pub baseline: PageBaseline,
    pub image: Image,
    pub clear_units: bool,
}

#[derive(Oper)]
#[oper(output = Page)]
pub struct ReadPage {
    pub comic_id: String,
    pub page_id: String,
}
