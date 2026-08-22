use std::{
    fmt::{
        Display,
        Formatter,
    },
    hash::{
        Hash,
        Hasher,
    },
};
use twox_hash::XxHash3_64;

const PREFIX: &str = "decal";

/// Opaque identifier representing a stable, hashed resource IRI.
#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub(crate) struct Iri(u64);

/// Trait for types that can be deterministically mapped to a resource IRI.
pub(crate) trait ResourceIri: Hash {
    /// Computes a stable IRI for the value based on its hash.
    ///
    /// The resulting IRI is deterministic and suitable for identifying
    /// resources across renders.
    ///
    /// # Returns
    /// - [`Iri`] derived from the hashed value.
    fn iri(&self) -> Iri {
        let mut hasher = XxHash3_64::with_seed(0);
        self.hash(&mut hasher);
        Iri(hasher.finish())
    }
}

impl Display for Iri {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{PREFIX}-{:x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Hash, Eq, PartialEq)]
    struct Stub(u32);

    impl ResourceIri for Stub {}

    #[test]
    fn renders_iri() {
        assert_eq!(Iri(0xdeadbeef).to_string(), "decal-deadbeef");
    }

    #[test]
    fn iri_is_deterministic() {
        assert_eq!(Stub(45).iri(), Stub(45).iri());
    }
}
