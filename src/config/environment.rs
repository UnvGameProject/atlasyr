use std::{
    env,
    io::{Error, ErrorKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppEnvironment {
    Development,
    Testing,
    Production,
}

impl AppEnvironment {
    pub(crate) fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let value = env::var("APP_ENV")?;

        Ok(parse_environment(&value)?)
    }

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Testing => "testing",
            Self::Production => "production",
        }
    }

    pub(crate) const fn reload_views(self) -> bool {
        matches!(self, Self::Development)
    }

    pub(crate) const fn view_debug_enabled(self) -> bool {
        !matches!(self, Self::Production)
    }

    pub(crate) const fn validate_views_at_startup(self) -> bool {
        !matches!(self, Self::Development)
    }
}

fn parse_environment(value: &str) -> Result<AppEnvironment, Error> {
    match value {
        "development" => Ok(AppEnvironment::Development),
        "testing" => Ok(AppEnvironment::Testing),
        "production" => Ok(AppEnvironment::Production),

        _ => Err(Error::new(
            ErrorKind::InvalidInput,
            "APP_ENV must name a supported application environment; \
             supported values: development, testing, production",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{AppEnvironment, parse_environment};

    #[test]
    fn accepts_development_environment() {
        assert_eq!(
            parse_environment("development").expect("environment should be supported"),
            AppEnvironment::Development,
        );
    }

    #[test]
    fn accepts_testing_environment() {
        assert_eq!(
            parse_environment("testing").expect("environment should be supported"),
            AppEnvironment::Testing,
        );
    }

    #[test]
    fn accepts_production_environment() {
        assert_eq!(
            parse_environment("production").expect("environment should be supported"),
            AppEnvironment::Production,
        );
    }

    #[test]
    fn rejects_unknown_environment() {
        assert!(parse_environment("staging").is_err());
    }

    #[test]
    fn only_development_reloads_views() {
        assert!(AppEnvironment::Development.reload_views());
        assert!(!AppEnvironment::Testing.reload_views());
        assert!(!AppEnvironment::Production.reload_views());
    }

    #[test]
    fn production_disables_view_debug_information() {
        assert!(AppEnvironment::Development.view_debug_enabled());
        assert!(AppEnvironment::Testing.view_debug_enabled());
        assert!(!AppEnvironment::Production.view_debug_enabled());
    }

    #[test]
    fn non_development_environments_validate_views_at_startup() {
        assert!(!AppEnvironment::Development.validate_views_at_startup());
        assert!(AppEnvironment::Testing.validate_views_at_startup());
        assert!(AppEnvironment::Production.validate_views_at_startup());
    }
}
