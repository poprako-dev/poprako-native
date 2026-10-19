use poprako_orchestra::Oper;

use crate::data::export::ExportSnapshot;

#[derive(Oper)]
#[oper(output = ExportSnapshot)]
pub struct ReadExportSnapshot {
    pub chapter_id: String,
}
