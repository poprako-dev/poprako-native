mod build_icon;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tauri embeds cached icon bytes; track the source assets to invalidate that cache.
    println!("cargo:rerun-if-changed=icons");

    build_icon::generate()?;

    println!("cargo:rerun-if-changed=migration");

    // The desktop binary embeds the frontend; rebuild when Vite changes its output.
    println!("cargo:rerun-if-changed=../dist");

    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "list_termbase_infos",
            "get_termbase",
            "export_termbase",
            "list_term_infos",
            "copy_term_target",
            "create_termbase",
            "choose_termbase_import",
            "import_termbase",
            "choose_termbase_append",
            "append_termbase",
            "update_termbase",
            "delete_termbase",
            "create_term",
            "update_term",
            "delete_term",
            "select_images",
            "select_image_folder",
            "get_image_import",
            "cancel_image_import",
            "confirm_image_import",
            "choose_archive_import",
            "get_archive_task",
            "cancel_archive_task",
            "acknowledge_archive_task",
            "confirm_archive_import",
            "export_archive",
            "reorder_pages",
            "remove_pages",
            "list_comic_infos",
            "create_comic",
            "update_comic_metadata",
            "delete_comic",
            "reorder_chapters",
            "move_chapter",
            "list_chapter_infos",
            "search_units",
            "replace_units",
            "get_chapter_detail",
            "create_chapter",
            "update_chapter_metadata",
            "delete_chapter",
            "get_page_editor",
            "save_page_units",
            "get_preference",
            "get_default_preference",
            "update_preference",
            "update_work_position",
            "get_image_resource",
            "get_image_tile",
            "release_image_resources",
            "request_exit",
            "get_write_session",
            "get_write_result",
            "acknowledge_write_result",
        ]),
    ))?;

    Ok(())
}
