#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UserRole {
    Operator,
    Viewer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Permission {
    ContactsRead,
    ContactsCreate,
    ContactsUpdate,
    ContactsDelete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PersistedRoleError;

impl UserRole {
    /// Convert the constrained database representation into the application's
    /// typed authorization role.
    ///
    /// Persisted role names are deliberately exact and case-sensitive. An
    /// unknown value fails closed rather than being interpreted as a
    /// lower-privilege or higher-privilege role.
    pub(super) fn from_persisted(value: &str) -> Result<Self, PersistedRoleError> {
        match value {
            "operator" => Ok(Self::Operator),
            "viewer" => Ok(Self::Viewer),
            _ => Err(PersistedRoleError),
        }
    }

    /// Central application policy for the current Contacts capability set.
    ///
    /// The database stores roles, not arbitrary permission strings.
    pub(super) fn allows(self, permission: Permission) -> bool {
        match self {
            Self::Operator => true,

            Self::Viewer => matches!(permission, Permission::ContactsRead),
        }
    }
}
