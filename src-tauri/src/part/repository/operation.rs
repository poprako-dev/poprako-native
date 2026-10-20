use poprako_orchestra::Oper;

use crate::data::chapter::ChapterMetadata;
use crate::data::editor::{PageEditor, SavePageUnits};
use crate::model::chapter::Chapter;

#[derive(Oper)]
#[oper(output = Chapter)]
pub struct CreateChapter {
    pub id: String,
    pub comic_id: String,
    pub baseline: Vec<String>,
    pub metadata: ChapterMetadata,
}

#[derive(Oper)]
#[oper(output = PageEditor)]
pub struct ReadPageEditor {
    pub chapter_id: String,
    pub page_id: String,
}

#[derive(Oper)]
#[oper(output = PageEditor)]
pub struct PersistPageUnits {
    pub snapshot: SavePageUnits,
}

#[derive(Oper)]
#[oper(output = Vec<crate::data::chapter::ChapterInfo>)]
pub struct ListChapterInfos {
    pub comic_id: String,
}

#[derive(Oper)]
#[oper(output = crate::data::chapter::ChapterDetail)]
pub struct ReadChapterDetail {
    pub chapter_id: String,
}

#[derive(Oper)]
#[oper(output = Chapter)]
pub struct UpdateChapterMetadata {
    pub chapter_id: String,
    pub metadata: ChapterMetadata,
}

#[derive(Oper)]
#[oper(output = ())]
pub struct DeleteChapter {
    pub chapter_id: String,
}

#[derive(Oper)]
#[oper(output = crate::model::preference::ApplicationPreference)]
pub struct ReadPreference;

#[derive(Oper)]
#[oper(output = crate::model::preference::ApplicationPreference)]
pub struct PersistPreference {
    pub baseline: crate::model::preference::ApplicationPreference,
    pub preference: crate::model::preference::ApplicationPreference,
}

#[derive(Oper)]
#[oper(output = crate::model::work_position::WorkPosition)]
pub struct PersistWorkPosition {
    pub position: crate::model::work_position::WorkPosition,
}
