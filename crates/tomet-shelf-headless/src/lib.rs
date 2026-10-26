//! Headless engine library surface for `tomet-shelf-headless`.
//!
//! Provides the primary API for building and serving Tomet books:
//! - [`book::build_book`]: Builds an entire book from a vault directory.
//! - [`run_dev_server`]: Runs a local development server with live reload.
//! - [`config::BookConfig`]: Configuration parser and models.
//! - [`PageRenderer`] / [`RendererFactory`]: Template-engine-agnostic page rendering traits.

pub mod assets;
pub mod book;
pub mod renderer;
pub mod serve;

// Companion subcrates re-exports
pub use tomet_shelf_config as config;
pub use tomet_shelf_document as document;
pub use tomet_shelf_render as render;
pub use tomet_shelf_route as route;
pub use tomet_shelf_search as search;

pub use assets::{
    StaticAsset, content_version, write_if_changed, write_static_assets,
};
pub use renderer::{
    Backlink, EntrySummary, MinimalRenderer, MinimalRendererFactory, PageRenderer, RendererFactory,
    SectionSummary,
};

pub use book::{BuildReport, DocFailure, build_book};
pub use config::BookConfig;
pub use serve::TometDevHandler;

use anyhow::Result;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Arc;

/// Run the local dev server with live reload: builds a `TometDevHandler`
/// (tomet-aware incremental rebuilds) and hands it to
/// `tomet_shelf_serve::run_dev_server`, which owns the actual file-watching,
/// static serving, and websocket reload signaling.
pub async fn run_dev_server(
    src_dir: PathBuf,
    out_dir: PathBuf,
    config: BookConfig,
    renderer_factory: Arc<dyn RendererFactory>,
    host: IpAddr,
    port: u16,
) -> Result<()> {
    let prefix = Some(config.build.clean_url_prefix().to_string());
    let handler = TometDevHandler::new(src_dir.clone(), out_dir.clone(), config, renderer_factory);
    tomet_shelf_serve::run_dev_server(src_dir, None, host, port, prefix, handler).await
}
