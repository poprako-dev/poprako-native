pub mod archive;
pub mod content;
pub mod error;
pub mod image;
pub mod image_import;
pub mod lifecycle;
pub mod page;
pub mod protocol;
pub mod save;
pub mod write;

use tauri_specta::{Builder, collect_commands};

#[must_use]
pub fn builder() -> Builder<tauri::Wry> {
    Builder::new().commands(collect_commands![
        image_import::select_images,
        image_import::select_image_folder,
        image_import::get_image_import,
        image_import::cancel_image_import,
        image_import::confirm_image_import,
        archive::choose_archive_import,
        archive::get_archive_task,
        archive::cancel_archive_task,
        archive::acknowledge_archive_task,
        archive::confirm_archive_import,
        archive::export_archive,
        page::reorder_pages,
        page::remove_pages,
        content::list_comic_infos,
        content::search_units,
        content::replace_units,
        content::get_comic_detail,
        content::create_comic,
        content::update_comic_metadata,
        content::delete_comic,
        content::get_page_editor,
        save::save_page_units,
        content::get_preference,
        content::get_default_preference,
        content::update_preference,
        content::update_work_position,
        image::get_image_resource,
        image::get_image_tile,
        image::release_image_resources,
        lifecycle::request_exit,
        write::get_write_session,
        write::get_write_result,
        write::acknowledge_write_result,
    ])
}
