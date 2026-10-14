use std::{
    env,
    io::{Error, ErrorKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthDriverKind {
    LocalTopcoat,
}

pub(crate) struct AuthConfig {
    driver: AuthDriverKind,
}

impl AuthConfig {
    pub(crate) fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let driver = env::var("AUTH_DRIVER")?;

        Ok(Self {
            driver: parse_driver(&driver)?,
        })
    }

    pub(crate) fn driver(&self) -> AuthDriverKind {
        self.driver
    }

    #[cfg(test)]
    pub(crate) fn local_topcoat_for_test() -> Self {
        Self {
            driver: AuthDriverKind::LocalTopcoat,
        }
    }
}

fn parse_driver(value: &str) -> Result<AuthDriverKind, Error> {
    match value {
        "local-topcoat" => Ok(AuthDriverKind::LocalTopcoat),

        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            "AUTH_DRIVER must name a supported authentication driver; \
             supported values: local-topcoat",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{AuthDriverKind, parse_driver};

    #[test]
    fn accepts_local_topcoat_driver() {
        assert_eq!(
            parse_driver("local-topcoat").expect("driver should be supported"),
            AuthDriverKind::LocalTopcoat,
        );
    }

    #[test]
    fn rejects_unimplemented_topcoat_native_driver() {
        assert!(parse_driver("topcoat-native").is_err());
    }

    #[test]
    fn rejects_unknown_driver() {
        assert!(parse_driver("anything-else").is_err());
    }
}
