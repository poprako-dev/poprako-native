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
            "search_units",
            "replace_units",
            "get_comic_detail",
            "create_comic",
            "update_comic_metadata",
            "delete_comic",
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
