use poprako_orchestra::Oper;

use crate::data::comic::{ChapterOrder, ComicInfo, ComicMetadata, MoveChapter, NewComic};
use crate::model::chapter::Chapter;
use crate::model::comic::Comic;

#[derive(Oper)]
#[oper(output = Vec<ComicInfo>)]
pub struct ListComicInfos {
    pub offset: u32,
    pub limit: u32,
}

#[derive(Oper)]
#[oper(output = Chapter)]
pub struct CreateComic {
    pub id: String,
    pub chapter_id: String,
    pub input: NewComic,
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
#[oper(output = ())]
pub struct ReorderChapters {
    pub input: ChapterOrder,
}

#[derive(Oper)]
#[oper(output = Chapter)]
pub struct RelocateChapter {
    pub input: MoveChapter,
}

#[derive(Oper)]
#[oper(output = Comic)]
pub struct ReadComic {
    pub comic_id: String,
}
