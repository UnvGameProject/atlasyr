#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::web) struct ViewName(&'static str);

impl ViewName {
    pub(super) const fn trusted(name: &'static str) -> Self {
        Self(name)
    }

    pub(super) const fn as_str(self) -> &'static str {
        self.0
    }
}
