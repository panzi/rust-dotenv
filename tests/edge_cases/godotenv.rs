pub const FIXTURE: &[(&str, &str)] = &[
    ("VAR2", ""),
    ("VAR5", "FOO  BAR"),
    ("VAR6", "FOO  BAR"),
    ("VAR9", "FOO\nBAR2=BAZ"),
    ("VAR12", "#COMMENT"),
    ("VAR13", "TEXT#COMMENT"),
    ("VAR14", "#NO COMMNET"),
    ("VAR15", "#NO COMMNET"),
    ("VAR16", "double quoted backslash:\\double quote:\"single quote:'newline:\ntab:tbackspace:bformfeed:fcarrige return:\runicode ä:u00e4"),
    ("VAR18", "no quote backslash:\\\\double quote:\\\"single quote:\\'newline:\\ntab:\\tbackspace:\\bformfeed:\\fcarrige return:\\runicode ä:\\u00e4"),
    ("VAR19", "FOO"),
    ("VAR20", "FOO\\nBAR"),
    ("VAR21", "FOO\nBAR"),
    ("VAR23", "double\nquoted"),
    ("VAR25", "double\nquoted"),
    ("VAR28", "single-quoted"),
    ("VAR29", "single-quoted"),
    ("VAR30", "single-quoted"),
    ("VAR31", "single-quoted"),
    ("VAR32", "single\nquoted"),
    ("VAR35", "FOO\" BAR BAZ\""),
    ("VAR37", "EXPORT!"),
    ("JSON1", "{\"foo\": \"bar \\n no quotes"),
    ("JSON3", "{\"foo\": \"bar \\n single quotes #\"}"),
    ("JSON4", "`{\"foo\": \"bar \\n backticks"),
    ("PRE_DEFINED", "not override"),
];
