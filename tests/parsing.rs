
static SIMPLE_SCENE: &str = r"$baba #keke&me>fofo
jiji>>>:m/3>>";

static FLAGS: &str = r"--background=#FFFFFF -let -b=0,3";

static VAR_TEST: &str = r"baba:m/3";

static ARG_FAIL: &str = r"me:m/2/invalid";

static VAR_FAIL: &str = r"me:dne";

#[test]
fn test_parsing() {
    dbg!(chilly::parser::parse(SIMPLE_SCENE).expect("failed to parse simple scene"));
    dbg!(chilly::parser::parse(FLAGS).expect("failed to parse flags"));
    dbg!(chilly::parser::parse(VAR_TEST).expect("failed to parse variant test"));
    eprintln!("{}", chilly::parser::parse(ARG_FAIL).expect_err("successfully parsed variant that had invalid argument"));
    eprintln!("{}", chilly::parser::parse(VAR_FAIL).expect_err("successfully parsed variant that doesn't exist"));
}
