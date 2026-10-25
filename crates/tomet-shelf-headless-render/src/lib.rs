pub mod assets;
pub mod document;
pub mod renderer;

pub use tomet_shelf_route as route;
pub use tomet_shelf_route as slug;

pub use assets::{
    ICON_ASSETS, LUCIDE_SPRITE, SIMPLE_SPRITE, StaticAsset, content_version, write_if_changed,
    write_static_assets,
};
pub use document::{
    HeroChipItem, InfoboxRowItem, ProcessedDoc, SectionTab, TocItem, enhance_internal_links,
    extract_workspace_config_blocks, process_parsed_document, process_parsed_document_with_blocks,
    strip_doc_extension,
};
pub use renderer::{
    Backlink, EntrySummary, MinimalRenderer, MinimalRendererFactory, PageRenderer, RendererFactory,
    SectionSummary,
};
pub use route::{
    DocRouteInput, RouteTable, clean_doc_slug, clean_segment, normalize_path, normalize_path_str,
};
