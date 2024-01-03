use sailfish::TemplateOnce;

use crate::config::ConfigTemplate;

pub const STATIC_CSS : &str = include_str!("../web/style.css");
pub const STATIC_JS : &str = include_str!("../web/infiniscroll.js");

#[derive(Debug, TemplateOnce)]
#[template(path = "index.stpl")]
pub struct IndexTemplate<'a> {
	root: &'a ConfigTemplate,
}

impl<'a> From<&'a ConfigTemplate> for IndexTemplate<'a> {
	fn from(value: &'a ConfigTemplate) -> Self {
		IndexTemplate { root: value }
	}
}
