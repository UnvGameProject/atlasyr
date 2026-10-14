use std::{
    env,
    error::Error,
    ffi::OsString,
    io::{self, IsTerminal, Read},
};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("create-user failed: {error}");

        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (email, role) = parse_arguments()?;

    let password = read_password_from_stdin()?;

    atlasyr::run_create_user(email, role, password).await?;

    println!("User created.");

    Ok(())
}

fn parse_arguments() -> Result<(String, String), io::Error> {
    let mut arguments = env::args_os();

    let _program = arguments.next();

    let email = arguments.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: create-user <email> <viewer|operator>",
        )
    })?;

    let role = arguments.next().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: create-user <email> <viewer|operator>",
        )
    })?;

    if arguments.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: create-user <email> <viewer|operator>",
        ));
    }

    Ok((
        os_string_to_utf8(email, "email must be valid UTF-8")?,
        os_string_to_utf8(role, "role must be valid UTF-8")?,
    ))
}

fn os_string_to_utf8(value: OsString, message: &'static str) -> Result<String, io::Error> {
    value
        .into_string()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, message))
}

fn read_password_from_stdin() -> Result<String, io::Error> {
    if io::stdin().is_terminal() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "password must be supplied through stdin; use ./create-user",
        ));
    }

    let mut password = String::new();

    io::stdin().read_to_string(&mut password)?;

    if password.ends_with('\n') {
        password.pop();

        if password.ends_with('\r') {
            password.pop();
        }
    }

    if password.contains('\n') || password.contains('\r') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "password input must contain exactly one line",
        ));
    }

    Ok(password)
}
