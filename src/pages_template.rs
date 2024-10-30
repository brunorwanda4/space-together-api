use minijinja::Environment;

pub fn pages_template() -> Result<Environment<'static>, String> {
    let mut env = Environment::new();

    env.add_template("layout.html", include_str!("./website/layout.html"))
        .map_err(|e| format!("Failed to add layout.html: {}", e))?;

    env.add_template("index.html", include_str!("./website/index.html"))
        .map_err(|e| format!("Failed to add index.html: {}", e))?;

    Ok(env)
}
