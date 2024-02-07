
static SIMPLE_SCENE: &str =
    r"baba keke&me>fofo jiji:red>>>:blue>>jiji";

#[test]
fn test_parsing() -> Result<(), Box<dyn std::error::Error>> {
    let parsed = chilly::parser::parse(SIMPLE_SCENE);
    if let Err(ref e) = parsed {
        println!("{e}");
        Err(e.clone())?;
    }
    let parsed = parsed.unwrap();
    println!("{parsed:?}");
    Ok(())
}
