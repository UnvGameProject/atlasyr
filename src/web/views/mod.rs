pub(in crate::web) mod contacts;

mod name;
mod registry;
mod rendered;
mod renderer;

pub(in crate::web) use name::ViewName;
pub(in crate::web) use registry::{HOME_INDEX, VIEW_DX_DEMO};
pub(in crate::web) use rendered::RenderedHtml;
pub(crate) use renderer::ViewRenderer;
