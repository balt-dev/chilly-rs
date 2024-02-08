
static SIMPLE_SCENE: &str = r"baba keke&me>fofo
jiji:red>>>:blue>>jiji";

#[test]
fn test_parsing() -> Result<(), Box<dyn std::error::Error>> {
    chilly::parser::parse(SIMPLE_SCENE)
        .map(|_| ())
        .map_err(|err| err.into())
}
