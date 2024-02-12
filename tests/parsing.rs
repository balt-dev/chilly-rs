
static SIMPLE_SCENE: &str = r"$baba #keke&me>fofo
jiji>>>:m/3>>";

static FLAGS: &str = r"--baba=you --keke=move --valueless -small -b=0,3";

static ESCAPED: &str = r"--flag\ name=flag\ value tile\ name";

static VAR_TEST: &str = r"baba:m/3";

static MAND_PASS: &str = r"baba:mand/3 keke:mand/3/2>>:";

static MAND_FAIL: &str = r"me:mand";

static ARG_FAIL: &str = r"me:mand/a";

static ARG_FAIL2: &str = r"me:m/2/invalid";

static VAR_FAIL: &str = r"me:dne";

#[test]
fn test_parsing() {
    dbg!(chilly::parser::parse(SIMPLE_SCENE).expect("failed to parse simple scene"));
    dbg!(chilly::parser::parse(FLAGS).expect("failed to parse flags"));
    dbg!(chilly::parser::parse(ESCAPED).expect("failed to parse escaped string"));
    dbg!(chilly::parser::parse(VAR_TEST).expect("failed to parse variant test"));
    dbg!(chilly::parser::parse(MAND_PASS).expect("failed to parse mandatory variant test"));
    eprintln!("{}", chilly::parser::parse(MAND_FAIL).expect_err("successfully parsed variant that had not enough arguments"));
    eprintln!("{}", chilly::parser::parse(ARG_FAIL).expect_err("successfully parsed variant that had invalid argument"));
    eprintln!("{}", chilly::parser::parse(ARG_FAIL2).expect_err("successfully parsed variant that had invalid argument"));
    eprintln!("{}", chilly::parser::parse(VAR_FAIL).expect_err("successfully parsed variant that does not exist"));
}
