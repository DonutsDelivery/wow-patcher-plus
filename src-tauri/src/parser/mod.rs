pub mod forum;
pub mod modules;
pub mod links;
pub mod dependencies;

pub use forum::{ForumParser, fetch_forum_post, fetch_forum_post_with_fallback, FORUM_URL, FORUM_URL_ALT};
pub use modules::{parse_modules, get_all_modules};
pub use links::extract_download_links;
pub use dependencies::{validate_module_selection, auto_select_dependencies, get_conflicts};
