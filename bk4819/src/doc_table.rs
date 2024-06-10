//! A macro to generate the register documentation table.

// this is just *awful*, but the table is useful, and it's also nice to
// automatically generate a test that the table is accurate.
macro_rules! doc_table {
    {$($row:literal => {$($($reg:ty)?,)*},)*} => {
        concat!(
            "## Register Map\n\n",
            crate::doc_table::doc_table!(@header),
            crate::doc_table::doc_table!(@sep),
            $(
                crate::doc_table::doc_table!(@row, $row => {$($($reg)?,)*}),
            )*
            "\n",
            // even with all lines hidden, this shows up as an empty box
            // I can't figure out how to hide it, or another way to do this
            // oh well. it's not that ugly.
            "```\n",
            $(
                crate::doc_table::doc_table!(@testrow, $row => {$($($reg)?,)*}),
            )*
            "```\n",
        )
    };

    (@header) => {"||0|1|2|3|4|5|6|7|8|9|a|b|c|d|e|f|\n"};

    (@sep) => {"|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|-|\n"};

    (@row, $row:literal => {$($($reg:ty)?,)*}) => {
        concat!(
            "|**", $row, "**|",
            $(
                crate::doc_table::doc_table!(@entry, $($reg)?), "|",
            )*
            "\n",
        )
    };

    (@entry,) => {
        "--"
    };

    (@entry, $reg:ty) => {
        concat!("[", stringify!($reg), "]")
    };

    (@testrow, $row:literal => {$($($reg:ty)?,)*}) => {
        concat!(
            "# use bk4819::registers::*;\n",
            "# {\n",
            "#     let addr = \"", $row, "\".trim_start_matches(\"0x\");\n",
            "#     let mut addr = u8::from_str_radix(addr, 16).unwrap();\n",
            $(
                crate::doc_table::doc_table!(@testentry, $($reg)?),
                "#     addr += 1;\n",
            )*
            "# }\n",
        )
    };

    (@testentry,) => {""};

    (@testentry, $reg:ty) => {
        concat!("#     assert_eq!(addr, ", stringify!($reg), "::ADDRESS);\n")
    };
}

pub(crate) use doc_table;
