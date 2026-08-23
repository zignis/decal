use cosmic_text::{
    FontSystem,
    SwashCache,
    fontdb::{
        ID,
        Source,
    },
};
use hashbrown::HashMap;
use smart_default::SmartDefault;
use std::sync::Arc;

pub(crate) const DEFAULT_FONT_FAMILY: &str = "sans-serif";
pub(crate) const BASE_FONT_SIZE: f32 = 16.0;
pub(crate) const BASE_LINE_HEIGHT: f32 = 22.0;

#[cfg(not(target_arch = "wasm32"))]
fn default_font_system() -> FontSystem {
    FontSystem::new()
}

#[cfg(target_arch = "wasm32")]
fn default_font_system() -> FontSystem {
    FontSystem::new_with_fonts(std::iter::empty())
}

/// The central font registry managing font loading, aliases, and resolution.
#[derive(Debug, SmartDefault)]
pub struct FontRegistry {
    /// The underlying [`FontSystem`] used for shaping and font lookup.
    #[default(default_font_system())]
    pub(crate) system: FontSystem,
    /// The mapping from user-defined aliases to loaded font IDs.
    pub(crate) aliases: HashMap<String, Vec<ID>>,
    /// The cache mapping aliases to resolved family names.
    pub(crate) alias_cache: HashMap<String, String>,
    /// The [`SwashCache`] used for rendering glyphs.
    #[default(SwashCache::new())]
    pub(crate) swash_cache: SwashCache,
    /// The default font family.
    #[default(DEFAULT_FONT_FAMILY)]
    pub(crate) default_family: &'static str,
}

impl FontRegistry {
    /// Creates a new font registry.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads a font from raw data and associates it with the provided `alias`.
    ///
    /// # Arguments
    /// - `alias`: The alias used to reference the font family.
    /// - `data`: The raw font data.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn load_font<T>(mut self, alias: &str, data: T) -> Self
    where
        T: Into<Vec<u8>>,
    {
        self.append_font(alias, data);
        self
    }

    /// Loads all available system fonts into the registry.
    ///
    /// # Note
    /// Calls `fontdb`'s [`load_system_fonts`] internally.
    ///
    /// # Returns
    /// - [`Self`]
    ///
    /// [`load_system_fonts`]: cosmic_text::fontdb::Database::load_system_fonts
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_system_fonts(mut self) -> Self {
        self.system.db_mut().load_system_fonts();
        self
    }

    /// Sets the default font family.
    ///
    /// # Arguments
    /// - `alias`: The default font family alias.
    ///
    /// # Returns
    /// - [`Self`]
    pub fn default_family(mut self, alias: &'static str) -> Self {
        self.default_family = alias;
        self
    }

    /// Registers a font in the underlying font database.
    ///
    /// # Arguments
    /// - `alias`: The alias used to reference the font family.
    /// - `data`: The raw font data.
    pub(crate) fn append_font<T>(&mut self, alias: &str, data: T)
    where
        T: Into<Vec<u8>>,
    {
        let source = Source::Binary(Arc::new(data.into()));
        let ids = self.system.db_mut().load_font_source(source);
        self.aliases
            .entry(alias.to_string())
            .or_default()
            .extend_from_slice(&ids);
    }

    /// Resolves and returns the default font family name.
    ///
    /// # Returns
    /// - The resolved font family name, or the built-in default family if
    ///   resolution fails.
    pub(crate) fn get_default_family(&mut self) -> String {
        self.resolve_family_name(self.default_family)
            .unwrap_or(DEFAULT_FONT_FAMILY.to_string())
    }

    /// Resolves a font family alias into a concrete family name.
    ///
    /// # Arguments
    /// - `alias`: The font family alias to resolve.
    ///
    /// # Returns
    /// - `Some(String)` containing the resolved family name.
    /// - `None` if the alias cannot be resolved.
    pub(crate) fn resolve_family_name(&mut self, alias: &str) -> Option<String> {
        let alias = alias.trim();
        let lower = alias.to_ascii_lowercase();

        if matches!(
            lower.as_str(),
            "sans-serif" | "serif" | "mono" | "monospace"
        ) {
            return Some(lower);
        }

        if let Some(name) = self.alias_cache.get(alias) {
            return Some(name.clone());
        }

        let id = self.aliases.get(alias)?.first()?;
        let face = self.system.db().face(*id)?;
        let (name, _) = face.families.first()?;
        self.alias_cache.insert(alias.to_string(), name.clone());

        Some(name.to_owned())
    }
}
