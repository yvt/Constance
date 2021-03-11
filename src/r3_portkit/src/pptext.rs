//! Conditional string literal generation

/// Define a macro that produces a string literal whose contents is revealed
/// or masked based on the current build configuration (`cfg!`).
///
/// # Examples
///
/// ```
/// #![feature(decl_macro)]
///
/// r3_portkit::pptext::pp_text_macro! {
///     macro get_text {
///         "endianness = "
///         if cfg!(target_endian = "little") { "little" } else { "big" } ", "
///         "running on "
///         if cfg!(unix) { "a unix-like system" } else { "an unknown kind of system" }
///     }
/// }
///
/// // `get_text!()` expands to a string literal that can be used in many
/// // places where a string literal is expected
/// const TEXT: &str = concat!(get_text!(), "\n");
///
/// assert_eq!(
///     TEXT,
///     format!(
///         "endianness = {}, running on {}\n",
///         if cfg!(target_endian = "little") { "little" } else { "big" },
///         if cfg!(unix) { "a unix-like system" } else { "an unknown kind of system" },
///     ),
/// );
/// ```
pub macro pp_text_macro {
    (
        $vis:vis macro $name:ident {
            $($text:tt)*
        }
    ) => {
        $crate::pptext::pp_text_macro! {
            @internal,
            text: [{ $($text)* }],
            free_idents: [{
                __pp_01 __pp_02 __pp_03 __pp_04
                __pp_05 __pp_06 __pp_07 __pp_08
                __pp_09 __pp_10 __pp_11 __pp_12
                __pp_13 __pp_14 __pp_15 __pp_16
            }],
            private_mod: [{}],
            macro_inner: [{}],
            define_macro: [{ $vis $name }],
        }
    },
    // -----------------------------------------------------------------
    // Each step processes the first element in the given `text`. This continues
    // until none are left.
    (
        @internal,
        // The text being munched
        text: [{
            $text:literal
            $($rest_text:tt)*
        }],
        // A pool of unique identifiers.
        free_idents: [{ $($free_idents:tt)* }],
        // The contents of the generated private module.
        private_mod: [{ $($private_mod:tt)* }],
        // The contents of the generated macro..
        macro_inner: [{ $($macro_inner:tt)* }],
        define_macro: [$define_macro:tt],
    ) => {
        $crate::pptext::pp_text_macro! {
            @internal,
            text: [{ $($rest_text)* }],
            free_idents: [{ $($free_idents)* }],
            private_mod: [{ $($private_mod)* }],
            macro_inner: [{
                $($macro_inner)*

                $text,
            }],
            define_macro: [$define_macro],
        }
    },

    (
        @internal,
        text: [{
            $($macro:ident)::+!($($params:tt)*)
            $($rest_text:tt)*
        }],
        free_idents: [{ $($free_idents:tt)* }],
        private_mod: [{ $($private_mod:tt)* }],
        macro_inner: [{ $($macro_inner:tt)* }],
        define_macro: [$define_macro:tt],
    ) => {
        $crate::pptext::pp_text_macro! {
            @internal,
            text: [{ $($rest_text)* }],
            free_idents: [{ $($free_idents)* }],
            private_mod: [{ $($private_mod)* }],
            macro_inner: [{
                $($macro_inner)*

                $($macro)::+!($($params)*),
            }],
            define_macro: [$define_macro],
        }
    },

    (
        @internal,
        text: [{
            if cfg!( $($cfg:tt)* ) {
                $($true_text:tt)*
            } else {
                $($false_text:tt)*
            }
            $($rest_text:tt)*
        }],
        free_idents: [{ $free_ident:ident $($rest_free_idents:tt)* }],
        private_mod: [{ $($private_mod:tt)* }],
        macro_inner: [{ $($macro_inner:tt)* }],
        define_macro: [{ $vis:vis $name:ident }],
    ) => {
        $crate::pptext::pp_text_macro! {
            @internal,
            text: [{ $($rest_text)* }],
            free_idents: [{ $($rest_free_idents)* }],
            private_mod: [{
                $($private_mod)*

                #[cfg($($cfg)*)]
                $crate::pptext::pp_text_macro! {
                    pub macro $free_ident { $($true_text)* }
                }

                #[cfg(not($($cfg)*))]
                $crate::pptext::pp_text_macro! {
                    pub macro $free_ident { $($false_text)* }
                }
            }],
            macro_inner: [{
                $($macro_inner)*

                // Refers to the macro in the generated private module
                $name::$free_ident!(),
            }],
            define_macro: [{ $vis $name }],
        }
    },

    (
        @internal,
        text: [{
            if cfg!( $($cfg:tt)* ) {
                $($true_text:tt)*
            }
            $($rest_text:tt)*
        }],
        free_idents: [$free_idents:tt],
        private_mod: [$private_mod:tt],
        macro_inner: [$macro_inner:tt],
        define_macro: [$define_macro:tt],
    ) => {
        $crate::pptext::pp_text_macro! {
            @internal,
            text: [{
                // Expand to if-else
                if cfg!( $($cfg)* ) {
                    $($true_text)*
                } else {}
                $($rest_text)*
            }],
            free_idents: [$free_idents],
            private_mod: [$private_mod],
            macro_inner: [$macro_inner],
            define_macro: [$define_macro],
        }
    },

    (
        @internal,
        text: [{}],
        free_idents: [{ $free_ident:ident $($rest_free_idents:tt)* }],
        private_mod: [{ $($private_mod:tt)* }],
        macro_inner: [{ $($macro_inner:tt)* }],
        define_macro: [{ $vis:vis $name:ident }],
    ) => {
        // No more text to munch, define the macro
        mod $name { $($private_mod)* }

        #[doc(hidden)]
        $vis macro $name () { concat!( $($macro_inner)* ) }
    },
}

/// Preprocessed `llvm_asm!`.
pub macro pp_llvm_asm {
    // -------------------------------------------------------------------
    // Munch the input until `:` or EOF is found
    (
        @internal,
        unprocessed: [{ : $($unprocessed:tt)* }],
        code: [{ $($code:tt)* }],
    ) => {{
        $crate::pptext::pp_llvm_asm!(
            @done,
            unprocessed: [{ : $($unprocessed)* }],
            code: [{ $($code)* }],
        );
    }},
    (
        @internal,
        unprocessed: [{}],
        code: [{ $($code:tt)* }],
    ) => {{
        $crate::pptext::pp_llvm_asm!(
            @done,
            unprocessed: [{}],
            code: [{ $($code)* }],
        );
    }},

    (
        @internal,
        unprocessed: [{ $fragment:tt $($unprocessed:tt)* }],
        code: [{ $($code:tt)* }],
    ) => {{
        $crate::pptext::pp_llvm_asm!(
            @internal,
            unprocessed: [{ $($unprocessed)* }],
            code: [{ $($code)* $fragment }],
        );
    }},
    // -------------------------------------------------------------------
    (
        @done,
        unprocessed: [{ $($unprocessed:tt)* }],
        code: [{ $($code:tt)* }],
    ) => {{
        $crate::pptext::pp_text_macro! {
            macro pp_llvm_asm_code { $($code)* }
        }
        llvm_asm!(pp_llvm_asm_code!() $($unprocessed)*);
    }},
    // -------------------------------------------------------------------
    // The entry point
    (
        // TODO: remove `$l:lit`
        $l:literal $($rest:tt)*
    ) => {
        $crate::pptext::pp_llvm_asm!(
            @internal,
            unprocessed: [{ $l $($rest)* }],
            code: [{}],
        );
    },
}

/// Preprocessed `asm!`.
///
/// # Examples
///
/// ```
/// #![feature(asm)]
/// #![feature(decl_macro)]
///
/// #[macro_export]  // work-around for mysterious macro hygienics behavior
/// macro_rules! the_ultimate_answer { () => { "42" }; }
///
/// // miri doesn't support inline assembly
/// #[cfg(not(miri))]
/// unsafe {
///     let output: usize;
///     r3_portkit::pptext::pp_asm!(
///         // The input fragments are simply concatenated. This means `mov`
///         // would be part of this line comment if it didn't include a line
///         // break.
///         "# hoge \n"
///         if cfg!(target_arch = "x86_64") {
///             "mov {}, " crate::the_ultimate_answer!()
///         }
///         if cfg!(any(target_arch = "aarch64", target_arch = "arm")) {
///             "mov {}, #" crate::the_ultimate_answer!()
///         }
///         if cfg!(any(target_arch = "riscv32", target_arch = "riscv64")) {
///             "li {}, " crate::the_ultimate_answer!()
///         },
///         out(reg) output
///     );
///     assert_eq!(output.to_string(), crate::the_ultimate_answer!());
/// }
/// ```
pub macro pp_asm {
    // -------------------------------------------------------------------
    // Munch the input until `,` or EOF is found
    (
        @internal,
        unprocessed: [{ , $($unprocessed:tt)* }],
        code: [{ $($code:tt)* }],
    ) => {{
        $crate::pptext::pp_asm!(
            @done,
            unprocessed: [{ , $($unprocessed)* }],
            code: [{ $($code)* }],
        );
    }},
    (
        @internal,
        unprocessed: [{}],
        code: [{ $($code:tt)* }],
    ) => {{
        $crate::pptext::pp_asm!(
            @done,
            unprocessed: [{}],
            code: [{ $($code)* }],
        );
    }},

    (
        @internal,
        unprocessed: [{ $fragment:tt $($unprocessed:tt)* }],
        code: [{ $($code:tt)* }],
    ) => {{
        $crate::pptext::pp_asm!(
            @internal,
            unprocessed: [{ $($unprocessed)* }],
            code: [{ $($code)* $fragment }],
        );
    }},
    // -------------------------------------------------------------------
    (
        @done,
        unprocessed: [{ $($unprocessed:tt)* }],
        code: [{ $($code:tt)* }],
    ) => {{
        $crate::pptext::pp_text_macro! {
            macro pp_asm_code { $($code)* }
        }
        asm!(pp_asm_code!() $($unprocessed)*);
    }},
    // -------------------------------------------------------------------
    // The entry point
    (
        // TODO: remove `$l:lit`
        $l:literal $($rest:tt)*
    ) => {
        $crate::pptext::pp_asm!(
            @internal,
            unprocessed: [{ $l $($rest)* }],
            code: [{}],
        );
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    macro hoge_gen() {
        "hoge"
    }

    #[test]
    fn test_pp_text() {
        pp_text_macro!(
            macro got {
                "hello"
                hoge_gen!()
                "foo1"
                if cfg!(any()) { "foo2" } else { "bar2" }
                if cfg!(any()) { "foo3" }
                if cfg!(all()) { "foo4" } else { "bar4" }
                if cfg!(all()) { "foo5" }

                if cfg!(any()) {
                    "foo6"
                    if cfg!(any()) { "hoge1" } else { "piyo1" }
                    if cfg!(all()) { "hoge2" } else { "piyo2" }
                    "foo7"
                } else {
                    "foo8"
                    if cfg!(any()) { "hoge3" } else { "piyo3" }
                    if cfg!(all()) { "hoge4" } else { "piyo4" }
                    "foo9"
                }
                if cfg!(any()) {
                    "foo10"
                    if cfg!(any()) { "hoge5" } else { "piyo5" }
                    if cfg!(all()) { "hoge6" } else { "piyo6" }
                    "foo11"
                }
                if cfg!(all()) {
                    "foo12"
                    if cfg!(any()) { "hoge7" } else { "piyo7" }
                    if cfg!(all()) { "hoge8" } else { "piyo8" }
                    "foo13"
                }
            }
        );

        let got: &str = got!();
        let expected = {
            extern crate std;
            use std::borrow::ToOwned;
            let mut x = "hellohogefoo1".to_owned();
            x += if cfg!(any()) { "foo2" } else { "bar2" };
            x += if cfg!(any()) { "foo3" } else { "" };
            x += if cfg!(all()) { "foo4" } else { "bar4" };
            x += if cfg!(all()) { "foo5" } else { "" };

            if cfg!(any()) {
                x += "foo6";
                x += if cfg!(any()) { "hoge1" } else { "piyo1" };
                x += if cfg!(all()) { "hoge2" } else { "piyo2" };
                x += "foo7";
            } else {
                x += "foo8";
                x += if cfg!(any()) { "hoge3" } else { "piyo3" };
                x += if cfg!(all()) { "hoge4" } else { "piyo4" };
                x += "foo9";
            }
            if cfg!(any()) {
                x += "foo10";
                x += if cfg!(any()) { "hoge5" } else { "piyo5" };
                x += if cfg!(all()) { "hoge6" } else { "piyo6" };
                x += "foo11";
            }
            if cfg!(all()) {
                x += "foo12";
                x += if cfg!(any()) { "hoge7" } else { "piyo7" };
                x += if cfg!(all()) { "hoge8" } else { "piyo8" };
                x += "foo13";
            }

            x
        };

        assert_eq!(got, expected);
    }
}
