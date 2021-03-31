(function() {var implementors = {};
implementors["humantime"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/time/struct.Duration.html\" title=\"struct core::time::Duration\">Duration</a>&gt; for <a class=\"struct\" href=\"humantime/struct.Duration.html\" title=\"struct humantime::Duration\">Duration</a>","synthetic":false,"types":["humantime::wrapper::Duration"]},{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/std/time/struct.SystemTime.html\" title=\"struct std::time::SystemTime\">SystemTime</a>&gt; for <a class=\"struct\" href=\"humantime/struct.Timestamp.html\" title=\"struct humantime::Timestamp\">Timestamp</a>","synthetic":false,"types":["humantime::wrapper::Timestamp"]}];
implementors["r3"] = [{"text":"impl&lt;System:&nbsp;<a class=\"trait\" href=\"r3/kernel/trait.Kernel.html\" title=\"trait r3::kernel::Kernel\">Kernel</a>, T:&nbsp;?<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;T&gt; for <a class=\"struct\" href=\"r3/hunk/struct.Hunk.html\" title=\"struct r3::hunk::Hunk\">Hunk</a>&lt;System, T&gt;","synthetic":false,"types":["r3::hunk::Hunk"]}];
implementors["regex_syntax"] = [{"text":"impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.slice.html\">[</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u8.html\">u8</a><a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.slice.html\">]</a>&gt; for <a class=\"struct\" href=\"regex_syntax/hir/literal/struct.Literal.html\" title=\"struct regex_syntax::hir::literal::Literal\">Literal</a>","synthetic":false,"types":["regex_syntax::hir::literal::Literal"]}];
implementors["staticvec"] = [{"text":"impl&lt;T, const N:&nbsp;usize&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;T&gt; for <a class=\"struct\" href=\"staticvec/struct.PushCapacityError.html\" title=\"struct staticvec::PushCapacityError\">PushCapacityError</a>&lt;T, N&gt;","synthetic":false,"types":["staticvec::errors::PushCapacityError"]},{"text":"impl&lt;const N:&nbsp;usize&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;str&gt; for <a class=\"struct\" href=\"staticvec/struct.StaticString.html\" title=\"struct staticvec::StaticString\">StaticString</a>&lt;N&gt;","synthetic":false,"types":["staticvec::string::StaticString"]},{"text":"impl&lt;const N:&nbsp;usize&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;[u8]&gt; for <a class=\"struct\" href=\"staticvec/struct.StaticString.html\" title=\"struct staticvec::StaticString\">StaticString</a>&lt;N&gt;","synthetic":false,"types":["staticvec::string::StaticString"]},{"text":"impl&lt;T, const N:&nbsp;usize&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.AsRef.html\" title=\"trait core::convert::AsRef\">AsRef</a>&lt;[T]&gt; for <a class=\"struct\" href=\"staticvec/struct.StaticVec.html\" title=\"struct staticvec::StaticVec\">StaticVec</a>&lt;T, N&gt;","synthetic":false,"types":["staticvec::StaticVec"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()