#![doc = include_str!("../README.md")]
//!
//! For an entry point to the library, check the docs of [`BubbleBath`] or [`clean`]
//!

#![forbid(rust_2018_idioms)]
#![deny(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]

use ahash::{AHashMap, AHashSet};
use lol_html::{
    html_content::{ContentType, Element, TextChunk},
    DocumentContentHandlers, ElementContentHandlers, HandlerResult, Selector, Settings,
};
use once_cell::sync::Lazy;
use slab::Slab;
use std::{borrow::Cow, cell::RefCell, fmt::Write, rc::Rc, str::FromStr};

#[doc(hidden)]
pub use ahash;

pub use lol_html::{errors::RewritingError, MemorySettings};

mod macros;

const ANCHOR_TAG_NAME: &str = "a";
const ANCHOR_HREF_ATTRIBUTE_NAME: &str = "href";

const IMAGE_TAG_NAME: &str = "img";
const IMAGE_SRC_ATTRIBUTE_NAME: &str = "src";

static GLOBAL_BUBBLE_BATH: Lazy<BubbleBath<'static>> = Lazy::new(BubbleBath::default);
static SELECT_ALL: Lazy<Selector> = Lazy::new(|| Selector::from_str("*").unwrap());

/// Clean provided HTML with a global [`BubbleBath`] instance, constructed using [`BubbleBath::default`]
///
/// ## Important
///
/// The global instance does *not* limit memory usage by default. If you need to limit memory usage, build your own [`BubbleBath`] instance
///
/// # Errors
///
/// See [`BubbleBath::clean`] documentation
#[inline]
pub fn clean(content: &str) -> Result<String, RewritingError> {
    GLOBAL_BUBBLE_BATH.clean(content)
}

#[inline]
fn clean_text(source: &str) -> String {
    let mut acc = String::with_capacity(source.len());

    for chr in source.chars() {
        let replacement = match chr {
            '<' => "&lt;",
            '>' => "&gt;",
            '\"' => "&quot;",
            '\'' => "&apos;",
            '`' => "&grave;",
            '/' => "&#47;",
            '&' => "&amp;",
            '=' => "&#61;",
            '\0' => "&#65533;",
            _ => {
                acc.push(chr);
                continue;
            }
        };

        acc.push_str(replacement);
    }
    acc
}

/// HTML sanitizer
///
/// `bubble-bath` is allow-list based, meaning all tags are by default cleaned.
///
/// `BubbleBath::default` provides a safe default
///
/// ## Implementation details
///
/// - We use `lol_html` as our underlying HTML processor
/// - Only absolute URLs (i.e. URLs with a scheme) are allowed. Relative links are discarded
pub struct BubbleBath<'a> {
    /// Attributes you want to keep on all tags
    pub allowed_generic_attributes: AHashSet<&'a str>,

    /// Tags you want to keep
    pub allowed_tags: AHashSet<&'a str>,

    /// Attributes you want to keep on a per-tag basis
    pub allowed_tag_attributes: AHashMap<&'a str, AHashSet<&'a str>>,

    /// Schemes you want to allow on URLs in anchor tags
    pub allowed_url_schemes: AHashSet<&'a str>,

    /// Memory settings for the underlying HTML transformer
    pub memory_settings: MemorySettings,

    /// Instead of removing tags (and potentially their content), escape the HTML instead and output them as raw text
    pub preserve_escaped: bool,

    /// Tags of which you want to remove the tag *and* the content of
    ///
    /// By default `bubble-bath` preserves the content of tags
    ///
    /// **Note**: Remember to put `<script>` and `<style>` tags in here (unless you 100% know what you are doing) since they are really damn evil!
    pub remove_content_tags: AHashSet<&'a str>,

    /// Attributes you want to set on a per-tag basis
    pub set_tag_attributes: AHashMap<&'a str, AHashMap<&'a str, &'a str>>,
}

impl BubbleBath<'_> {
    #[inline]
    fn clean_attributes(&self, element: &mut Element<'_, '_>, tag_name: &str) {
        let allowed_attributes = self.allowed_tag_attributes.get(tag_name);

        let mut remove_attributes = Vec::with_capacity(element.attributes().len());
        for attribute in element.attributes() {
            let attribute_name = attribute.name();

            if self
                .allowed_generic_attributes
                .contains(attribute_name.as_str())
            {
                continue;
            }

            if let Some(allowed_attributes) = allowed_attributes {
                if allowed_attributes.contains(attribute_name.as_str()) {
                    continue;
                }
            }

            remove_attributes.push(attribute_name);
        }

        for attribute_name in remove_attributes {
            element.remove_attribute(&attribute_name);
        }
    }

    #[inline]
    fn clean_link(&self, element: &mut Element<'_, '_>, attribute_name: &str) {
        let Some(raw_url) = element.get_attribute(attribute_name) else {
            return;
        };

        let Some((scheme, _rest)) = raw_url.split_once("://") else {
            element.remove_attribute(attribute_name);
            return;
        };

        if !self.allowed_url_schemes.contains(scheme) {
            element.remove_attribute(attribute_name);
        }
    }

    #[inline]
    fn delete_element(&self, element: &mut Element<'_, '_>, tag_name: &str) {
        if self.preserve_escaped {
            let start_tag = element.start_tag();

            let mut formatted = String::new();
            let _ = write!(formatted, "<{tag_name}");

            for attribute in start_tag.attributes() {
                let _ = write!(formatted, " {}=\"{}\"", attribute.name(), attribute.value());
            }

            if start_tag.self_closing() {
                formatted.push_str(" />");
            } else {
                formatted.push('>');
            }

            start_tag.replace(&formatted, ContentType::Text);

            if let Some(handlers) = element.end_tag_handlers() {
                handlers.push(Box::new(move |end_tag| {
                    let tag_name = end_tag.name();
                    let content = format!("</{tag_name}>");
                    end_tag.replace(&content, ContentType::Text);

                    Ok(())
                }));
            }
        } else {
            element.remove_and_keep_content();
        }
    }

    fn element_handler(
        &self,
        element: &mut Element<'_, '_>,
        unclosed_tags: Rc<RefCell<Slab<String>>>,
    ) -> HandlerResult {
        let tag_name = element.tag_name();

        if self.remove_content_tags.contains(tag_name.as_str()) {
            element.remove();
            return Ok(());
        }

        if !self.allowed_tags.contains(tag_name.as_str()) {
            self.delete_element(element, &tag_name);
            return Ok(());
        }

        match tag_name.as_str() {
            ANCHOR_TAG_NAME => self.clean_link(element, ANCHOR_HREF_ATTRIBUTE_NAME),
            IMAGE_TAG_NAME => self.clean_link(element, IMAGE_SRC_ATTRIBUTE_NAME),
            _ => (),
        }

        self.clean_attributes(element, &tag_name);

        if let Some(set_attributes) = self.set_tag_attributes.get(tag_name.as_str()) {
            for (name, value) in set_attributes {
                element.set_attribute(name, value)?;
            }
        }

        // Manually balance the tags if they aren't self-closing
        if !element.is_self_closing() {
            let unclosed_tag_idx = {
                let mut unclosed_tags = unclosed_tags.borrow_mut();
                unclosed_tags.insert(tag_name)
            };

            if let Some(end_tag_handlers) = element.end_tag_handlers() {
                end_tag_handlers.push(Box::new(move |_end_tag| {
                    unclosed_tags.borrow_mut().remove(unclosed_tag_idx);
                    Ok(())
                }));
            }
        }

        Ok(())
    }

    #[inline]
    fn text_handler(chunk: &mut TextChunk<'_>) {
        *chunk.as_mut_str() = clean_text(chunk.as_str());
    }

    /// Clean the provided HTML content
    ///
    /// # Errors
    ///
    /// - The HTML rewriter ran out of memory
    /// - The HTML parser ran into an ambiguous state (in this case you should just discard the text instead of trying to fix it)
    /// - The name of an attribute you put into the `set_tag_attributes` hashmap is invalid
    #[inline]
    pub fn clean(&self, content: &str) -> Result<String, RewritingError> {
        let unclosed_tags = Rc::new(RefCell::new(Slab::new()));

        let text_handler = |chunk: &mut TextChunk<'_>| {
            Self::text_handler(chunk);
            Ok(())
        };

        let document_content_handlers = vec![DocumentContentHandlers {
            text: Some(Box::new(text_handler)),
            end: Some(Box::new(|document_end| {
                let unclosed_tags = unclosed_tags.borrow();
                for (_key, content) in unclosed_tags.iter() {
                    let formatted = format!("</{content}>");
                    document_end.append(&formatted, ContentType::Html);
                }

                Ok(())
            })),
            ..DocumentContentHandlers::default()
        }];

        let element_content_handlers = vec![(
            Cow::Borrowed(&*SELECT_ALL),
            ElementContentHandlers::default()
                .element(|element| self.element_handler(element, unclosed_tags.clone()))
                .text(text_handler),
        )];

        lol_html::rewrite_str(
            content,
            Settings {
                document_content_handlers,
                element_content_handlers,
                memory_settings: MemorySettings {
                    preallocated_parsing_buffer_size: self
                        .memory_settings
                        .preallocated_parsing_buffer_size,
                    max_allowed_memory_usage: self.memory_settings.max_allowed_memory_usage,
                },
                ..Settings::default()
            },
        )
    }
}

impl Default for BubbleBath<'static> {
    #[allow(clippy::too_many_lines)]
    fn default() -> Self {
        // Safe defaults taken from ammonia
        #[rustfmt::skip]
        let allowed_tags = hashset![
            "a", "abbr", "acronym", "area", "article", "aside", "b", "bdi",
            "bdo", "blockquote", "br", "caption", "center", "cite", "code",
            "col", "colgroup", "data", "dd", "del", "details", "dfn", "div",
            "dl", "dt", "em", "figcaption", "figure", "footer", "h1", "h2",
            "h3", "h4", "h5", "h6", "header", "hgroup", "hr", "i", "img",
            "ins", "kbd", "li", "map", "mark", "nav", "ol", "p", "pre",
            "q", "rp", "rt", "rtc", "ruby", "s", "samp", "small", "span",
            "strike", "strong", "sub", "summary", "sup", "table", "tbody",
            "td", "th", "thead", "time", "tr", "tt", "u", "ul", "var", "wbr",
        ];
        let allowed_generic_attributes = hashset!["lang", "title"];
        let allowed_tag_attributes = hashmap![
            "a" => hashset![
                "href", "hreflang"
            ],
            "bdo" => hashset![
                "dir"
            ],
            "blockquote" => hashset![
                "cite"
            ],
            "col" => hashset![
                "align", "char", "charoff", "span"
            ],
            "colgroup" => hashset![
                "align", "char", "charoff", "span"
            ],
            "del" => hashset![
                "cite", "datetime"
            ],
            "hr" => hashset![
                "align", "size", "width"
            ],
            "img" => hashset![
                "align", "alt", "height", "src", "width"
            ],
            "ins" => hashset![
                "cite", "datetime"
            ],
            "ol" => hashset![
                "start"
            ],
            "q" => hashset![
                "cite"
            ],
            "table" => hashset![
                "align", "char", "charoff", "summary"
            ],
            "tbody" => hashset![
                "align", "char", "charoff"
            ],
            "td" => hashset![
                "align", "char", "charoff", "colspan", "headers", "rowspan"
            ],
            "tfoot" => hashset![
                "align", "char", "charoff"
            ],
            "th" => hashset![
                "align", "char", "charoff", "colspan", "headers", "rowspan", "scope"
            ],
            "thead" => hashset![
                "align", "char", "charoff"
            ],
            "tr" => hashset![
                "align", "char", "charoff"
            ],
        ];
        let allowed_url_schemes = hashset![
            "bitcoin",
            "ftp",
            "ftps",
            "geo",
            "http",
            "https",
            "im",
            "irc",
            "ircs",
            "magnet",
            "mailto",
            "mms",
            "mx",
            "news",
            "nntp",
            "openpgp4fpr",
            "sip",
            "sms",
            "smsto",
            "ssh",
            "tel",
            "url",
            "webcal",
            "wtai",
            "xmpp",
        ];
        let remove_content_tags = hashset!["script", "style"];
        let set_tag_attributes = hashmap![
            ANCHOR_TAG_NAME => hashmap![
                "rel" => "noopener noreferrer",
            ],
        ];

        Self {
            allowed_tags,
            allowed_generic_attributes,
            allowed_tag_attributes,
            allowed_url_schemes,
            memory_settings: MemorySettings::default(),
            preserve_escaped: false,
            remove_content_tags,
            set_tag_attributes,
        }
    }
}
