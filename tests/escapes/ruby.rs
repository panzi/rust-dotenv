pub const FIXTURE: &[(&str, &str)] = &[
    ("BASIC", "\\r,\\n,t,v,f,a,b"),
    ("BACKSLASH", "\\"),
    ("QUOTES", "\",'"),
    ("SINGLE_QUOTED1", "\\'"),
    ("SINGLE_QUOTED2", "\\'"),
    ("OCT1", "90090"),
    ("OCT2", "53053"),
    ("OCT3", "157143164"),
    ("OCT4", "015701430164"),
    ("HEX", "x48x45x58x2E"),
    ("UTF16", "u00e4"),
    ("UTF16_PAIR", "uD83DuDE03"),
    ("UTF32_6", "U01F603"),
    ("UTF32_8", "U0001F603"),
    ("NAMED1", "u{Latin Capital Letter O with macron}"),
    ("NAMED2", "u{LATIN CAPITAL LETTER O WITH MACRON}"),
    ("NAMED3", "u{LATIN_CAPITAL_LETTER_O_WITH_MACRON}"),
    ("UNKNOWN", "/,z, "),
    ("ESCAPED_NEWLINE", "\n"),
];
