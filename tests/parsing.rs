
static SIMPLE_SCENE: &str = r"$baba #keke&me>fofo
jiji>>>:m/3>>";

static FLAGS: &str = r"--baba=you --keke=move --valueless -small -b=0,3";

static ESCAPED: &str = r"--flag\ name=flag\ value tile\ name";

static VAR_TEST: &str = r"baba:m/3";

#[test]
fn test_parsing() {
    dbg!(chilly::parser::parse(SIMPLE_SCENE).expect("failed to parse simple scene"));
    dbg!(chilly::parser::parse(FLAGS).expect("failed to parse flags"));
    dbg!(chilly::parser::parse(ESCAPED).expect("failed to parse escaped string"));
    dbg!(chilly::parser::parse(VAR_TEST).expect("failed to parse variant test"));
}
