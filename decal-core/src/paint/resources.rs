use super::Resource;
use hashbrown::HashMap;
use std::fmt::{
    Display,
    Formatter,
};

/// Collection managing a deduplicated set of render resources with stable
/// indexing.
#[derive(Debug, Clone, Default)]
pub(crate) struct Resources {
    index_map: HashMap<Resource, usize>,
    resources: Vec<Resource>,
}

impl Resources {
    /// Returns the index of the resource, inserting it if it does not already
    /// exist.
    ///
    /// # Arguments
    /// - `resource`: The [`Resource`] to look up or insert.
    ///
    /// # Returns
    /// - The index associated with the resource.
    pub(crate) fn get_or_add_resource(&mut self, resource: Resource) -> usize {
        if let Some(idx) = self.index_map.get(&resource) {
            return *idx;
        }

        let idx = self.resources.len();
        self.index_map.insert(resource.clone(), idx);
        self.resources.push(resource);

        idx
    }

    /// Returns `true` if no resources have been added.
    ///
    /// # Returns
    /// - `true` when the resource collection is empty.
    pub(crate) fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }

    pub(crate) fn inner(&self) -> &[Resource] {
        &self.resources
    }
}

impl Display for Resources {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for resource in &self.resources {
            resource.fmt(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::LinearGradient;

    #[test]
    fn adds_a_single_resource() {
        let mut resources = Resources::default();
        let res = Resource::LinearGradient(LinearGradient::default());
        let idx = resources.get_or_add_resource(res.clone());

        assert_eq!(idx, 0);
        assert!(!resources.is_empty());
        assert_eq!(resources.inner().len(), 1);
        assert_eq!(resources.inner()[0], res);
    }

    #[test]
    fn handles_duplicate_resource() {
        let mut resources = Resources::default();
        let res = Resource::LinearGradient(LinearGradient::default());
        let idx1 = resources.get_or_add_resource(res.clone());
        let idx2 = resources.get_or_add_resource(res.clone());

        assert_eq!(idx1, idx2);
        assert_eq!(resources.inner().len(), 1);
    }

    #[test]
    fn adds_multiple_resources() {
        let mut resources = Resources::default();
        let res1 = Resource::LinearGradient(LinearGradient::new().x1(0.5));
        let res2 = Resource::LinearGradient(LinearGradient::new().x1(0.7));
        let idx1 = resources.get_or_add_resource(res1.clone());
        let idx2 = resources.get_or_add_resource(res2.clone());

        assert_ne!(idx1, idx2);
        assert_eq!(resources.inner().len(), 2);
        assert_eq!(resources.inner()[0], res1);
        assert_eq!(resources.inner()[1], res2);
    }
}
