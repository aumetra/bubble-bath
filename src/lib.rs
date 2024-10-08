#![doc = include_str!("../README.md")]
//!
//! For an entry point to the library, check the docs of [`BubbleBath`] or [`clean`]
//!

use ahash::{HashMap, HashSet};
use lol_html::{
    errors::RewritingError,
    html_content::{Comment, ContentType, DocumentEnd, Element, TextChunk},
    DocumentContentHandlers, ElementContentHandlers, HandlerResult, HtmlRewriter, Selector,
    Settings,
};
use once_cell::sync::Lazy;
use slab::Slab;
use std::{borrow::Cow, cell::RefCell, fmt::Write, iter, rc::Rc, str::FromStr};
use thiserror::Error;

pub use lol_html::MemorySettings;

mod macros;

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
pub fn clean(content: &str) -> Result<String, Error> {
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

/// Potential errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// The rewriting of the HTML content failed
    #[error(transparent)]
    Rewriting(#[from] RewritingError),
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
    pub allowed_generic_attributes: HashSet<&'a str>,

    /// Tags you want to keep
    pub allowed_tags: HashSet<&'a str>,

    /// Attributes you want to keep on a per-tag basis
    pub allowed_tag_attributes: HashMap<&'a str, HashSet<&'a str>>,

    /// Schemes you want to allow on URLs in anchor tags
    pub allowed_url_schemes: HashSet<&'a str>,

    /// Clean certain attributes on tags as if they are URLs
    pub clean_url_attributes: HashMap<&'a str, HashSet<&'a str>>,

    /// Memory settings for the underlying HTML transformer
    pub memory_settings: MemorySettings,

    /// Instead of removing tags (and potentially their content), escape the HTML instead and output them as raw text
    pub preserve_escaped: bool,

    /// Tags of which you want to remove the tag *and* the content of
    ///
    /// By default `bubble-bath` preserves the content of tags
    ///
    /// **Note**: Remember to put `<script>` and `<style>` tags in here (unless you 100% know what you are doing) since they are really damn evil!
    pub remove_content_tags: HashSet<&'a str>,

    /// Attributes you want to set on a per-tag basis
    pub set_tag_attributes: HashMap<&'a str, HashMap<&'a str, &'a str>>,
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

    #[inline]
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

        self.clean_attributes(element, &tag_name);

        if let Some(set_attributes) = self.set_tag_attributes.get(tag_name.as_str()) {
            for (name, value) in set_attributes {
                element.set_attribute(name, value)?;
            }
        }

        if let Some(attributes) = self.clean_url_attributes.get(tag_name.as_str()) {
            for name in attributes {
                self.clean_link(element, name);
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
    fn count_unclosed_opening_tags<B>(counter: &mut usize, input: B)
    where
        B: AsRef<[u8]>,
    {
        let bytes = input.as_ref();

        let opening_tags = bytecount::count(bytes, b'<');
        let closing_tags = bytecount::count(bytes, b'>');

        *counter = counter.saturating_add(opening_tags);
        *counter = counter.saturating_sub(closing_tags);
    }

    #[inline]
    fn subtract_opening_tags<B>(counter: &mut usize, input: B)
    where
        B: AsRef<[u8]>,
    {
        let mut tmp_counter = 0;
        Self::count_unclosed_opening_tags(&mut tmp_counter, input);

        *counter = counter.saturating_sub(tmp_counter);
    }

    #[inline]
    fn comment_handler(comment: &mut Comment<'_>, opening_tags: &RefCell<usize>) {
        Self::subtract_opening_tags(&mut opening_tags.borrow_mut(), comment.text());
        comment.remove();
    }

    #[inline]
    fn text_handler(chunk: &mut TextChunk<'_>, opening_tags: &RefCell<usize>) {
        Self::subtract_opening_tags(&mut opening_tags.borrow_mut(), chunk.as_str());
        *chunk.as_mut_str() = clean_text(chunk.as_str());
    }

    /// Clean HTML in a streaming fashion
    ///
    /// # Errors
    ///
    /// - The HTML rewriter ran out of memory
    /// - The HTML parser ran into an ambiguous state (in this case you should just discard the text instead of trying to fix it)
    /// - The name of an attribute you put into the `set_tag_attributes` hashmap is invalid
    #[inline]
    pub fn clean_streaming<'a, I, S>(&self, input: I, sink: S) -> Result<(), Error>
    where
        I: Iterator<Item = &'a [u8]>,
        S: FnMut(&[u8]),
    {
        let unclosed_tags = Rc::new(RefCell::new(Slab::new()));
        let opening_tags = RefCell::new(0);

        let comment_handler = |comment: &mut Comment<'_>| {
            Self::comment_handler(comment, &opening_tags);
            Ok(())
        };
        let document_end_handler = |document_end: &mut DocumentEnd<'_>| {
            let unclosed_tags = unclosed_tags.borrow();
            for (_key, content) in unclosed_tags.iter() {
                let formatted = format!("</{content}>");
                document_end.append(&formatted, ContentType::Html);
            }

            Ok(())
        };
        let text_handler = |chunk: &mut TextChunk<'_>| {
            Self::text_handler(chunk, &opening_tags);
            Ok(())
        };

        let document_content_handlers = vec![DocumentContentHandlers::default()
            .comments(comment_handler)
            .text(text_handler)
            .end(document_end_handler)];

        // Don't ask me why we need this. This is dumb and I don't like it.
        // It's required so the compiler recognizes that our closure, indeed, implements the handler trait.
        #[inline(always)]
        fn bounds_assertion<T>(uwu: T) -> T
        where
            T: FnMut(&mut Element<'_, '_>) -> HandlerResult,
        {
            uwu
        }

        let element_content_handlers = vec![(
            Cow::Borrowed(&*SELECT_ALL),
            ElementContentHandlers::default().element(bounds_assertion(|element| {
                self.element_handler(element, unclosed_tags.clone())
            })),
        )];

        let settings = Settings {
            document_content_handlers,
            element_content_handlers,
            memory_settings: MemorySettings {
                preallocated_parsing_buffer_size: self
                    .memory_settings
                    .preallocated_parsing_buffer_size,
                max_allowed_memory_usage: self.memory_settings.max_allowed_memory_usage,
            },
            ..Settings::default()
        };

        let mut rewriter = HtmlRewriter::new(settings, sink);

        for chunk in input {
            Self::count_unclosed_opening_tags(&mut opening_tags.borrow_mut(), chunk);

            rewriter.write(chunk)?;
        }

        let opening_tags = *opening_tags.borrow();
        for _ in 0..opening_tags {
            rewriter.write(&[b'>'])?;
        }

        rewriter.end()?;

        Ok(())
    }

    /// Clean the provided HTML content
    ///
    /// # Errors
    ///
    /// - The output of the HTML transformer was not valid UTF-8
    ///
    /// Check [`Self::clean_streaming`] for additional errors
    #[inline]
    pub fn clean(&self, content: &str) -> Result<String, Error> {
        let mut acc = Vec::with_capacity(content.len());
        self.clean_streaming(iter::once(content.as_bytes()), |out| {
            acc.extend_from_slice(out);
        })?;

        // SAFETY: Since the input is a string slice, we can be confident that it is valid UTF-8.
        // We also buffered the entirety of the output into the accumulator.
        //
        // According to [this comment](https://github.com/cloudflare/lol-html/issues/200#issuecomment-1829731640),
        // `lol_html` always outputs the data in the same encoding it was supplied in.
        //
        // Meaning, since we have the entire output accumulated and the source encoding is valid UTF-8,
        // this byte vector is, indeed, valid UTF-8.
        #[allow(unsafe_code)]
        Ok(unsafe { String::from_utf8_unchecked(acc) })
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
        let clean_url_attributes = hashmap![
            "a" => hashset!["href"],
            "img" => hashset!["src"],
            "link" => hashset!["href"],
        ];
        let remove_content_tags = hashset!["script", "style"];
        let set_tag_attributes = hashmap![
            "a" => hashmap![
                "rel" => "noopener noreferrer",
            ],
        ];

        Self {
            allowed_tags,
            allowed_generic_attributes,
            allowed_tag_attributes,
            allowed_url_schemes,
            clean_url_attributes,
            memory_settings: MemorySettings::default(),
            preserve_escaped: false,
            remove_content_tags,
            set_tag_attributes,
        }
    }
}
