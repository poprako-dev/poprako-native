use poprako_orchestra::Oper;

use crate::data::comic::ComicMetadata;
use crate::data::editor::{PageEditor, SavePageUnits};
use crate::model::comic::Comic;

#[derive(Oper)]
#[oper(output = Comic)]
pub struct CreateComic {
    pub id: String,
    pub metadata: ComicMetadata,
}

#[derive(Oper)]
#[oper(output = PageEditor)]
pub struct ReadPageEditor {
    pub comic_id: String,
    pub page_id: String,
}

#[derive(Oper)]
#[oper(output = PageEditor)]
pub struct PersistPageUnits {
    pub snapshot: SavePageUnits,
}

#[derive(Oper)]
#[oper(output = Vec<crate::data::comic::ComicInfo>)]
pub struct ListComicInfos {
    pub offset: u32,
    pub limit: u32,
}

#[derive(Oper)]
#[oper(output = crate::data::comic::ComicDetail)]
pub struct ReadComicDetail {
    pub comic_id: String,
}

#[derive(Oper)]
#[oper(output = Comic)]
pub struct UpdateComicMetadata {
    pub comic_id: String,
    pub metadata: ComicMetadata,
}

#[derive(Oper)]
#[oper(output = ())]
pub struct DeleteComic {
    pub comic_id: String,
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
