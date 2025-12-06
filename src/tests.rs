use crate::{parse, parse_owned, Bytes};
use crate::{parser::*, HTMLTag, Node};

fn force_as_tag<'a, 'b>(actual: &'a Node<'b>) -> &'a HTMLTag<'b> {
    match actual {
        Node::Tag(t) => t,
        _ => panic!("Failed to force tag as Node::Tag (got {:?})", actual),
    }
}

#[test]
fn outer_html() {
    let dom = parse(
        "abc <p>test<span>a</span></p> def",
        ParserOptions::default(),
    )
    .unwrap();
    let parser = dom.parser();

    let tag = force_as_tag(dom.children()[1].get(parser).unwrap());

    assert_eq!(tag.outer_html(parser), "<p>test<span>a</span></p>");
}

#[test]
fn outer_html_void_elements() {
    const HTML_INPUT: &str = r#"<html><head></head><body><img src=""><br><hr></body></html>"#;
    let vdom = parse(HTML_INPUT, ParserOptions::default()).unwrap();
    assert_eq!(
        r#"<html><head></head><body><img src=""><br><hr></body></html>"#,
        vdom.outer_html()
    );
}

#[test]
fn inner_html() {
    let dom = parse(
        "abc <p>test<span>a</span></p> def",
        ParserOptions::default(),
    )
    .unwrap();
    let parser = dom.parser();

    let tag = force_as_tag(dom.children()[1].get(parser).unwrap());

    assert_eq!(tag.inner_html(parser), "test<span>a</span>");
}

#[test]
fn children_len() {
    let dom = parse(
        "<!-- element 1 --><div><div>element 3</div></div>",
        ParserOptions::default(),
    )
    .unwrap();
    assert_eq!(dom.children().len(), 2);
}

#[test]
fn get_element_by_id_default() {
    let dom = parse(
        "<div></div><p id=\"test\"></p><p></p>",
        ParserOptions::default(),
    )
    .unwrap();
    let parser = dom.parser();

    let tag = dom.get_element_by_id("test").expect("Element not present");

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.outer_html(parser), "<p id=\"test\"></p>")
}

#[test]
fn get_element_by_id_tracking() {
    let dom = parse(
        "<div></div><p id=\"test\"></p><p></p>",
        ParserOptions::default().track_ids(),
    )
    .unwrap();
    let parser = dom.parser();

    let tag = dom.get_element_by_id("test").expect("Element not present");

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.outer_html(parser), "<p id=\"test\"></p>")
}

#[test]
fn get_element_by_class_name_default() {
    let dom = parse(
        "<div></div><p class=\"a b\">hey</p><p></p>",
        ParserOptions::default(),
    )
    .unwrap();

    let tag = dom.get_elements_by_class_name("a").next().unwrap();

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_text(dom.parser()), "hey");
}

#[test]
fn get_element_by_class_name_tracking() {
    let dom = parse(
        "<div></div><p class=\"a b\">hey</p><p></p>",
        ParserOptions::default().track_ids(),
    )
    .unwrap();

    let tag = dom.get_elements_by_class_name("a").next().unwrap();

    let el = force_as_tag(tag.get(dom.parser()).unwrap());

    assert_eq!(el.inner_text(dom.parser()), "hey");
}

#[test]
fn html5() {
    let dom = parse("<!DOCTYPE html> hello", ParserOptions::default()).unwrap();

    assert_eq!(dom.version(), Some(HTMLVersion::HTML5));
    assert_eq!(dom.children().len(), 1)
}

#[test]
fn ignore_void_closing_tags() {
    let input = r#"
        <head>
            <base href='single_quoted_item'></base>
            <link rel="stylesheet" type="text/css" href="non-exising"/>
        </head>
    "#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let head_tag = force_as_tag(dom.children()[1].get(dom.parser()).unwrap());

    let base_tag = force_as_tag(head_tag.children().top()[1].get(dom.parser()).unwrap());
    let link_tag = force_as_tag(head_tag.children().top()[3].get(dom.parser()).unwrap());

    assert_eq!(head_tag.name(), "head");
    assert_eq!(base_tag.name(), "base");
    assert_eq!(link_tag.name(), "link");
}

#[test]
pub fn children_mut() {
    let input = "<head><p>Replace me</p> World</head>";

    let mut dom = parse(input, Default::default()).unwrap();
    let children = dom.children();
    let child = children[0]
        .clone()
        .get_mut(dom.parser_mut())
        .unwrap()
        .as_tag_mut()
        .unwrap();

    let mut children = child.children_mut();
    let top = children.top_mut();
    let handle = top[0];
    let node = handle.get_mut(dom.parser_mut()).unwrap();
    *node = Node::Raw("Hello".into());

    assert_eq!(dom.outer_html(), "<head>Hello World</head>");
}

#[test]
fn nested_inner_text() {
    let dom = parse(
        "<p>hello <p>nested element</p></p>",
        ParserOptions::default(),
    )
    .unwrap();
    let parser = dom.parser();

    let el = force_as_tag(dom.children()[0].get(parser).unwrap());

    assert_eq!(el.inner_text(parser), "hello nested element");
}

#[test]
fn owned_dom() {
    let owned_dom = {
        let input = String::from("<p id=\"test\">hello</p>");

        unsafe { parse_owned(input, ParserOptions::default()).unwrap() }
    };

    let dom = owned_dom.get_ref();
    let parser = dom.parser();

    let el = force_as_tag(dom.children()[0].get(parser).unwrap());

    assert_eq!(el.inner_text(parser), "hello");
}

#[test]
fn move_owned() {
    let input = String::from("<p id=\"test\">hello</p>");

    let guard = unsafe { parse_owned(input, ParserOptions::default()).unwrap() };

    fn move_me<T>(p: T) -> T {
        p
    }

    let guard = std::thread::spawn(|| guard).join().unwrap();
    let guard = move_me(guard);

    let dom = guard.get_ref();
    let parser = dom.parser();

    let el = force_as_tag(dom.children()[0].get(parser).unwrap());

    assert_eq!(el.inner_text(parser), "hello");
}

#[test]
fn with() {
    let input = r#"<p>hello <span>whats up</span></p>"#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();

    let tag = dom
        .nodes()
        .iter()
        .find(|x| x.as_tag().is_some_and(|x| x.name() == "span"));

    assert_eq!(
        tag.map(|tag| tag.inner_text(parser)),
        Some("whats up".into())
    )
}

#[test]
fn abrupt_attributes_stop() {
    let input = r#"<p "#;
    parse(input, ParserOptions::default()).unwrap();
}

#[test]
fn dom_nodes() {
    let input = r#"<p><p><a>nested</a></p></p>"#;
    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let element = dom
        .nodes()
        .iter()
        .find(|x| x.as_tag().is_some_and(|x| x.name().eq("a")));

    assert_eq!(element.map(|x| x.inner_text(parser)), Some("nested".into()));
}

#[test]
fn fuzz() {
    // Some tests that would previously panic or end in an infinite loop
    // We don't need to assert anything here, just see that they finish
    parse("J\x00<", ParserOptions::default()).unwrap();
    parse("<!J", ParserOptions::default()).unwrap();
    parse("<=/Fy<=/", Default::default()).unwrap();

    // Miri is too slow... :(
    let count = if cfg!(miri) { 100usize } else { 10000usize };

    parse(&"<p>".repeat(count), ParserOptions::default()).unwrap();
}

#[test]
fn mutate_dom() {
    let input = r#"<img src="test.png" />"#;
    let mut dom = parse(input, ParserOptions::default()).unwrap();

    let mut selector = dom.query_selector("[src]").unwrap();
    let handle = selector.next().unwrap();

    let parser = dom.parser_mut();

    let el = handle.get_mut(parser).unwrap();
    let tag = el.as_tag_mut().unwrap();
    let attr = tag.attributes_mut();
    let bytes = attr.get_mut("src").flatten().unwrap();
    bytes.set("world.png").unwrap();

    assert_eq!(attr.get("src"), Some(Some(&"world.png".into())));
}

mod simd {
    // These tests make sure that SIMD functions do the right thing

    #[test]
    fn matches_case_insensitive_test() {
        assert!(crate::simd::matches_case_insensitive(b"hTmL", *b"html"));
        assert!(!crate::simd::matches_case_insensitive(b"hTmLs", *b"html"));
        assert!(!crate::simd::matches_case_insensitive(b"hTmy", *b"html"));
        assert!(!crate::simd::matches_case_insensitive(b"/Tmy", *b"html"));
    }

    #[test]
    fn string_search() {
        assert_eq!(crate::simd::find(b"a", b' '), None);
        assert_eq!(crate::simd::find(b"", b' '), None);
        assert_eq!(crate::simd::find(b"a ", b' '), Some(1));
        assert_eq!(crate::simd::find(b"abcd ", b' '), Some(4));
        assert_eq!(crate::simd::find(b"ab cd ", b' '), Some(2));
        assert_eq!(crate::simd::find(b"abcdefgh ", b' '), Some(8));
        assert_eq!(crate::simd::find(b"abcdefghi ", b' '), Some(9));
        assert_eq!(crate::simd::find(b"abcdefghi", b' '), None);
        assert_eq!(crate::simd::find(b"abcdefghiabcdefghi .", b' '), Some(18));
        assert_eq!(crate::simd::find(b"abcdefghiabcdefghi.", b' '), None);

        let count = if cfg!(miri) { 500usize } else { 1000usize };

        let long = "a".repeat(count) + "b";
        assert_eq!(crate::simd::find(long.as_bytes(), b'b'), Some(count));
    }

    #[test]
    fn string_search_3() {
        const NEEDLE: [u8; 3] = [b'a', b'b', b'c'];

        assert_eq!(crate::simd::find3(b"e", NEEDLE), None);
        assert_eq!(crate::simd::find3(b"a", NEEDLE), Some(0));
        assert_eq!(crate::simd::find3(b"ea", NEEDLE), Some(1));
        assert_eq!(crate::simd::find3(b"ef", NEEDLE), None);
        assert_eq!(crate::simd::find3(b"ef a", NEEDLE), Some(3));
        assert_eq!(crate::simd::find3(b"ef g", NEEDLE), None);
        assert_eq!(crate::simd::find3(b"ef ghijk", NEEDLE), None);
        assert_eq!(crate::simd::find3(b"ef ghijkl", NEEDLE), None);
        assert_eq!(crate::simd::find3(b"ef ghijkla", NEEDLE), Some(9));
        assert_eq!(crate::simd::find3(b"ef ghiajklm", NEEDLE), Some(6));
        assert_eq!(crate::simd::find3(b"ef ghibjklm", NEEDLE), Some(6));
        assert_eq!(crate::simd::find3(b"ef ghicjklm", NEEDLE), Some(6));
        assert_eq!(crate::simd::find3(b"ef ghijklmnopqrstua", NEEDLE), Some(18));
        assert_eq!(crate::simd::find3(b"ef ghijklmnopqrstub", NEEDLE), Some(18));
        assert_eq!(crate::simd::find3(b"ef ghijklmnopqrstuc", NEEDLE), Some(18));
        assert_eq!(crate::simd::find3(b"ef ghijklmnopqrstu", NEEDLE), None);
    }

    #[test]
    #[rustfmt::skip]
    fn search_non_ident() {
        assert_eq!(crate::simd::search_non_ident(b"this-is-a-very-long-identifier<"), Some(30));
        assert_eq!(crate::simd::search_non_ident(b"0123456789Abc_-<"), Some(15));
        assert_eq!(crate::simd::search_non_ident(b"0123456789Abc-<"), Some(14));
        assert_eq!(crate::simd::search_non_ident(b"0123456789Abcdef_-<"), Some(18));
        assert_eq!(crate::simd::search_non_ident(b""), None);
        assert_eq!(crate::simd::search_non_ident(b"short"), None);
        assert_eq!(crate::simd::search_non_ident(b"short_<"), Some(6));
        assert_eq!(crate::simd::search_non_ident(b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_"), None);
        assert_eq!(crate::simd::search_non_ident(b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_<"), Some(64));
        assert_eq!(crate::simd::search_non_ident(b"0123456789ab<defghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_<"), Some(12));

        // Test empty and very short strings (fallback path).
        assert_eq!(crate::simd::search_non_ident(b""), None);
        assert_eq!(crate::simd::search_non_ident(b"a"), None);
        assert_eq!(crate::simd::search_non_ident(b"ab"), None);
        assert_eq!(crate::simd::search_non_ident(b"abc"), None);
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmno"), None); // 15 bytes (just under SIMD threshold)
        assert_eq!(crate::simd::search_non_ident(b"<"), Some(0));
        assert_eq!(crate::simd::search_non_ident(b"a<"), Some(1));
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmn<"), Some(14)); // 15 bytes

        // Test exactly 16 bytes (single SIMD iteration).
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnop"), None); // All ident
        assert_eq!(crate::simd::search_non_ident(b"<bcdefghijklmnop"), Some(0)); // Non-ident at position 0
        assert_eq!(crate::simd::search_non_ident(b"a<cdefghijklmnop"), Some(1)); // Non-ident at position 1
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmno<"), Some(15)); // Non-ident at position 15 (last byte of chunk)

        // Test 17-31 bytes (one SIMD iteration + fallback).
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnopq"), None);
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnop<"), Some(16)); // Non-ident at position 16 (first byte of fallback)
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnopq<"), Some(17));
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnopqrstuvwxyz12345"), None); // 31 bytes
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnopqrstuvwxyz1234<"), Some(30));

        // Test exactly 32 bytes (two SIMD iterations).
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnopqrstuvwxyz123456"), None); // All ident
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnopqrstuvwxyz12345<"), Some(31));
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnop<rstuvwxyz123456"), Some(16)); // Non-ident at start of 2nd chunk

        // Test 48 bytes (three SIMD iterations).
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJKL"), None);
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnopqrstuvwxyz0123456789ABCDEFGHIJK<"), Some(47));
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnopqrstuvwxyz01234<6789ABCDEFGHIJKL"), Some(31)); // Non-ident in 2nd chunk
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnopqrstuvwxyz0123456789ABCDEFG<IJK<"), Some(43)); // First non-ident in 3rd chunk

        // Test all valid identifier characters.
        assert_eq!(crate::simd::search_non_ident(b"0123456789"), None); // All digits
        assert_eq!(crate::simd::search_non_ident(b"abcdefghijklmnopqrstuvwxyz"), None); // All lowercase
        assert_eq!(crate::simd::search_non_ident(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"), None); // All uppercase
        assert_eq!(crate::simd::search_non_ident(b"_"), None); // Underscore
        assert_eq!(crate::simd::search_non_ident(b"-"), None); // Hyphen
        assert_eq!(crate::simd::search_non_ident(b"azAZ09-_"), None); // Mix of all valid types

        // Test actual non-identifier characters.
        assert_eq!(crate::simd::search_non_ident(b"<"), Some(0)); // 0x3C
        assert_eq!(crate::simd::search_non_ident(b">"), Some(0)); // 0x3E
        assert_eq!(crate::simd::search_non_ident(b"@"), Some(0)); // 0x40 (just before 'A')
        assert_eq!(crate::simd::search_non_ident(b"["), Some(0)); // 0x5B (just after 'Z')
        assert_eq!(crate::simd::search_non_ident(b"`"), Some(0)); // 0x60 (just before 'a')
        assert_eq!(crate::simd::search_non_ident(b"{"), Some(0)); // 0x7B (just after 'z')
        assert_eq!(crate::simd::search_non_ident(b" "), Some(0)); // Space
        assert_eq!(crate::simd::search_non_ident(b"="), Some(0)); // Equals

        // Test valid identifier characters that might seem like they shouldn't be ('/', ':', and '+'
        // are valid).
        assert_eq!(crate::simd::search_non_ident(b"/"), None); // '/' IS an identifier
        assert_eq!(crate::simd::search_non_ident(b":"), None); // ':' IS an identifier
        assert_eq!(crate::simd::search_non_ident(b"+"), None); // '+' IS an identifier

        // Test non-identifiers in the middle of valid identifiers.
        assert_eq!(crate::simd::search_non_ident(b"abc<def"), Some(3));
        assert_eq!(crate::simd::search_non_ident(b"abc>def"), Some(3));
        assert_eq!(crate::simd::search_non_ident(b"abc@def"), Some(3));
        assert_eq!(crate::simd::search_non_ident(b"abc[def"), Some(3));
        assert_eq!(crate::simd::search_non_ident(b"abc`def"), Some(3));
        assert_eq!(crate::simd::search_non_ident(b"abc{def"), Some(3));

        // Test non-identifier at each position in first 16-byte chunk.
        assert_eq!(crate::simd::search_non_ident(b"<234567890123456"), Some(0));
        assert_eq!(crate::simd::search_non_ident(b"0<34567890123456"), Some(1));
        assert_eq!(crate::simd::search_non_ident(b"01<4567890123456"), Some(2));
        assert_eq!(crate::simd::search_non_ident(b"012<567890123456"), Some(3));
        assert_eq!(crate::simd::search_non_ident(b"0123<67890123456"), Some(4));
        assert_eq!(crate::simd::search_non_ident(b"01234<7890123456"), Some(5));
        assert_eq!(crate::simd::search_non_ident(b"012345<890123456"), Some(6));
        assert_eq!(crate::simd::search_non_ident(b"0123456<90123456"), Some(7));
        assert_eq!(crate::simd::search_non_ident(b"01234567<0123456"), Some(8));
        assert_eq!(crate::simd::search_non_ident(b"012345678<123456"), Some(9));
        assert_eq!(crate::simd::search_non_ident(b"0123456789<23456"), Some(10));
        assert_eq!(crate::simd::search_non_ident(b"0123456789a<3456"), Some(11));
        assert_eq!(crate::simd::search_non_ident(b"0123456789ab<456"), Some(12));
        assert_eq!(crate::simd::search_non_ident(b"0123456789abc<56"), Some(13));
        assert_eq!(crate::simd::search_non_ident(b"0123456789abcd<6"), Some(14));
        assert_eq!(crate::simd::search_non_ident(b"0123456789abcde<"), Some(15));

        // Test special HTML/XML characters that are common non-identifiers.
        assert_eq!(crate::simd::search_non_ident(b"tag<"), Some(3));
        assert_eq!(crate::simd::search_non_ident(b"tag>"), Some(3));
        assert_eq!(crate::simd::search_non_ident(b"tag "), Some(3)); // Space
        assert_eq!(crate::simd::search_non_ident(b"tag="), Some(3)); // Equals
        assert_eq!(crate::simd::search_non_ident(b"tag\""), Some(3)); // Quote
        assert_eq!(crate::simd::search_non_ident(b"tag'"), Some(3)); // Single quote
        assert_eq!(crate::simd::search_non_ident(b"tag/"), None);
        assert_eq!(crate::simd::search_non_ident(b"tag:"), None);
        assert_eq!(crate::simd::search_non_ident(b"tag+"), None);

        // Test long strings with non-identifier at various positions.
        let long_ident = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_";
        assert_eq!(crate::simd::search_non_ident(long_ident), None);

        // 64 bytes, all identifiers.
        let mut buf = [b'a'; 64];
        assert_eq!(crate::simd::search_non_ident(&buf), None);

        // Non-identifier at position 63.
        buf[63] = b'<';
        assert_eq!(crate::simd::search_non_ident(&buf), Some(63));

        // Non-identifier at position 32 (start of 3rd chunk).
        buf[63] = b'a';
        buf[32] = b'<';
        assert_eq!(crate::simd::search_non_ident(&buf), Some(32));
    }
}

mod bytes {
    use crate::bytes::*;

    #[test]
    fn from_str() {
        let x = Bytes::from("hello");
        assert_eq!(x.as_bytes(), b"hello");
    }

    #[test]
    fn from_bytes() {
        let x = Bytes::from(b"hello" as &[u8]);
        assert_eq!(x.as_bytes(), b"hello");
    }

    #[test]
    fn as_bytes_borrowed() {
        let xb = Bytes::from(b"hello" as &[u8]);
        assert_eq!(xb.as_bytes_borrowed(), Some(b"hello" as &[u8]));

        let mut xc = xb.clone();
        xc.set("test2").unwrap();
        assert_eq!(xc.as_bytes_borrowed(), None);
    }

    #[test]
    fn as_utf8_str() {
        assert_eq!(Bytes::from("hello").as_utf8_str(), "hello");
    }

    #[test]
    fn clone_shallow() {
        // cloning a borrowed slice does not deep-clone
        let x = Bytes::from("hello");
        let xp = x.as_ptr();

        let y = x.clone();
        let yp = y.as_ptr();

        assert_eq!(xp, yp);
    }

    #[test]
    fn drop_old_owned() {
        let mut x = Bytes::from("");
        x.set("test").unwrap();
        x.set("test2").unwrap();
    }

    #[test]
    fn clone_owned_deep() {
        let mut x = Bytes::from("");
        x.set("hello").unwrap();
        let xp = x.as_ptr();

        let y = x.clone();
        let yp = y.as_ptr();

        assert_eq!(x, y);
        assert_ne!(xp, yp);
    }

    #[test]
    fn empty() {
        let _x = Bytes::new();
    }

    #[test]
    fn empty_set() {
        let mut x = Bytes::new();
        x.set("hello").unwrap();
    }

    #[test]
    fn set() {
        let mut x = Bytes::from("hello");
        let xp = x.as_ptr();

        x.set("world").unwrap();
        let xp2 = x.as_ptr();

        // check that the changes are reflected
        assert_eq!(x.as_bytes(), b"world");

        // pointer must be different now as the call to `set` should cause an allocation
        assert_ne!(xp, xp2);
    }

    #[test]
    fn clone_deep() {
        let x = Bytes::from("hello");
        let xp = x.as_ptr();

        let mut y = x.clone();
        y.set("world").unwrap();
        let yp = y.as_ptr();

        assert_ne!(xp, yp);
    }

    #[test]
    fn into_owned_bytes() {
        let mut x1 = Bytes::new();
        x1.set("hello").unwrap(); // &str

        let mut x2 = x1.clone();
        x2.set(b"world" as &[u8]).unwrap(); // &[u8]

        let mut x3 = x1.clone();
        x3.set(vec![0u8, 1, 2, 3, 4]).unwrap(); // Vec<u8>

        let mut x4 = x1.clone();
        x4.set(vec![0u8, 1, 2, 3, 4].into_boxed_slice()).unwrap(); // Box<[u8]>

        let mut x5 = x1.clone();
        x5.set(String::from("Tests are important")).unwrap(); // String
    }
}

#[test]
fn valueless_attribute() {
    // https://github.com/y21/tl/issues/11
    let input = r#"
        <a id="u54423">
            <iframe allowfullscreen></iframe>
        </a>
    "#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let element = dom.get_element_by_id("u54423");

    assert!(element.is_some());
}

#[test]
fn non_ident_attr_chars() {
    // Test that non-identifier characters in attribute position don't cause infinite loops.
    // These are edge cases where invalid HTML has special chars where an attribute name should be.
    let test_cases = [
        r#"<button " >"#,
        r#"<button "">"#,
        r#"<div @ >"#,
        r#"<span ! foo>"#,
        r#"<p # >"#,
        r#"<a = >"#,
        r#"<div < >"#,
    ];

    for input in test_cases {
        let _ = parse(input, ParserOptions::default());
    }
}

#[test]
fn unquoted() {
    // https://github.com/y21/tl/issues/12
    let input = r#"
        <a id=u54423>Hello World</a>
    "#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let element = dom.get_element_by_id("u54423");

    assert_eq!(
        element.and_then(|x| x.get(parser).map(|x| x.inner_text(parser))),
        Some("Hello World".into())
    );
}

#[test]
fn unquoted_href() {
    // https://github.com/y21/tl/issues/12
    let input = r#"
        <a id=u54423 href=https://www.google.com>Hello World</a>
    "#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();
    let element = dom.get_element_by_id("u54423");

    assert_eq!(
        element.and_then(|x| x.get(parser).map(|x| x
            .as_tag()
            .unwrap()
            .attributes()
            .get("href")
            .flatten()
            .unwrap()
            .try_as_utf8_str()
            .unwrap()
            .to_string())),
        Some("https://www.google.com".into())
    );
}

#[test]
fn unquoted_self_closing() {
    // https://github.com/y21/tl/issues/12
    let input = r#"
        <a id=u54423 />
    "#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let element = dom.get_element_by_id("u54423");

    assert!(element.is_some());

    // According to MDN, if there's no space between an unquoted attribute and the closing tag,
    // the slash is treated as part of the attribute value.
    let input = r#"
        <a id=u54423/>
    "#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let element = dom.get_element_by_id("u54423/");

    assert!(element.is_some());
}

mod query_selector {
    use super::*;
    #[test]
    fn query_selector_simple() {
        let input = "<div><p class=\"hi\">hello</p></div>";
        let dom = parse(input, ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let mut selector = dom.query_selector(".hi").unwrap();
        let el = force_as_tag(selector.next().and_then(|x| x.get(parser)).unwrap());

        assert_eq!(dom.nodes().len(), 3);
        assert_eq!(el.inner_text(parser), "hello");
    }

    #[test]
    fn tag_query_selector() {
        // empty
        let dom = parse("<p></p>", ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let selector = dom.nodes()[0]
            .as_tag()
            .unwrap()
            .query_selector(parser, "div.z")
            .unwrap();
        assert_eq!(selector.count(), 0);

        // one child
        let dom = parse(
            r#"<p><div class="z">PASS</div></p>"#,
            ParserOptions::default(),
        )
        .unwrap();
        let parser = dom.parser();
        let mut selector = dom.nodes()[0]
            .as_tag()
            .unwrap()
            .query_selector(parser, "div.z")
            .unwrap();
        assert_eq!(selector.clone().count(), 1);
        assert_eq!(
            selector
                .next()
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser),
            "PASS"
        );

        // nested
        let dom = parse(
            r#"<p><div class="z"><div class="y">PASS</div></div></p>"#,
            ParserOptions::default(),
        )
        .unwrap();
        let parser = dom.parser();
        let mut selector = dom.nodes()[0]
            .as_tag()
            .unwrap()
            .query_selector(parser, "div.y")
            .unwrap();
        assert_eq!(selector.clone().count(), 1);
        assert_eq!(
            selector
                .next()
                .unwrap()
                .get(parser)
                .unwrap()
                .inner_text(parser),
            "PASS"
        );
    }

    #[test]
    fn query_selector_with_quote() {
        let input = r#"<div><meta property="og:title" content="hello" /></div>"#;
        let dom = parse(input, ParserOptions::default()).unwrap();
        let parser = dom.parser();
        let node_option = dom
            .query_selector(r#"meta[property="og:title"]"#)
            .and_then(|mut iter| iter.next());
        let value = node_option.map(|node| {
            node.get(parser)
                .unwrap()
                .as_tag()
                .unwrap()
                .attributes()
                .get("content")
                .flatten()
                .unwrap()
                .try_as_utf8_str()
                .unwrap()
                .to_string()
        });

        assert_eq!(value, Some("hello".to_string()));
    }
}

#[test]
fn nodes_order() {
    let input = r#"
    <p>test</p><div><span>test2</span></div>
    "#
    .trim();
    let dom = parse(input, Default::default()).unwrap();
    let nodes = dom.nodes();

    // 5 nodes in total
    assert_eq!(nodes.len(), 5);

    // First node is <p>
    assert_eq!(&nodes[0].as_tag().unwrap()._name, "p");
    // Second node is inner text of <p>: test
    assert_eq!(nodes[1].as_raw().unwrap().as_bytes(), b"test");
    // Third node is <div>
    assert_eq!(&nodes[2].as_tag().unwrap()._name, "div");
    // Fourth node is inner <span> node
    assert_eq!(&nodes[3].as_tag().unwrap()._name, "span");
    // Fifth node is inner text of <span>: test2
    assert_eq!(nodes[4].as_raw().unwrap().as_bytes(), b"test2");
}

#[test]
fn comment() {
    let dom = parse("<!-- test -->", Default::default()).unwrap();
    let nodes = dom.nodes();
    assert_eq!(nodes.len(), 1);
    assert_eq!(
        nodes[0].as_comment().unwrap().as_utf8_str(),
        "<!-- test -->"
    );
}

#[test]
fn tag_all_children() {
    fn assert_len(input: &str, len: usize) {
        let dom = parse(input, Default::default()).unwrap();
        let el = dom.nodes()[0].as_tag().unwrap();
        assert_eq!(el.children().all(dom.parser()).len(), len);
    }

    fn assert_last(input: &str, last: &str) {
        let dom = parse(input, Default::default()).unwrap();
        let el = dom.nodes()[0].as_tag().unwrap();
        assert_eq!(
            el.children()
                .all(dom.parser())
                .last()
                .unwrap()
                .inner_text(dom.parser()),
            last
        );
    }

    assert_len(r#"<div></div>"#, 0);
    assert_len(r#"<div>a</div>"#, 1);
    assert_len(r#"<div><p></p></div>"#, 1);
    assert_len(r#"<div><p>a</p></div>"#, 2);
    assert_len(r#"<div><p><span></span></p></div>"#, 2);
    assert_len(r#"<div><p><span>a</span></p></div>"#, 3);

    assert_last(r#"<div>a</div>"#, "a");
    assert_last(r#"<div><p>a</p></div>"#, "a");
    assert_last(r#"<div>b<p>a</p></div>"#, "a");
    assert_last(r#"<div>b<p><span>a</span></p></div>"#, "a");
}

#[test]
fn assert_length() {
    fn assert_len(input: &str, selector: &str, len: usize) {
        let dom = parse(input, Default::default()).unwrap();
        let el = dom.nodes()[0].as_tag().unwrap();
        let query = el.query_selector(dom.parser(), selector).unwrap();
        assert_eq!(query.count(), len);
    }

    assert_len("<div></div>", "a", 0);
    assert_len("<div><a></a></div>", "a", 1);
    assert_len("<div><a><a></a></a></div>", "a", 2);
    assert_len("<div><a><span></span></a></div>", "span", 1);
}

#[test]
fn self_closing_no_child() {
    let dom = parse("<br /><p>test</p>", Default::default()).unwrap();
    let nodes = dom.nodes();
    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[0].as_tag().unwrap()._children.len(), 0);
    assert_eq!(nodes[0].as_tag().unwrap().raw(), "<br />");
}

#[test]
fn insert_attribute_owned() {
    // https://github.com/y21/tl/issues/27
    let mut attr = Attributes::new();
    let style = "some style".to_string();
    attr.insert("style", Some(Bytes::try_from(style).unwrap()));
    assert_eq!(attr.get("style"), Some(Some(&"some style".into())));
}

#[test]
fn boundaries() {
    // https://github.com/y21/tl/issues/25
    let dom = parse("<div><p>haha</p></div>", Default::default()).unwrap();
    let span = dom.nodes()[1].as_tag().unwrap();
    let boundary = span.boundaries(dom.parser());
    assert_eq!(boundary, (5, 15));
}

#[test]
fn attributes_remove_inner_html() {
    let mut dom = parse(
        "<span contenteditable=\"true\">testing</a>",
        Default::default(),
    )
    .unwrap();

    dom.nodes_mut()[0]
        .as_tag_mut()
        .unwrap()
        .attributes_mut()
        .remove_value("contenteditable");

    assert_eq!(dom.outer_html(), "<span contenteditable>testing</span>");

    dom.nodes_mut()[0]
        .as_tag_mut()
        .unwrap()
        .attributes_mut()
        .remove("contenteditable");

    assert_eq!(dom.outer_html(), "<span>testing</span>");
}

#[test]
fn tag_raw() {
    let input = "<p>abcd</p>";

    let vdom = parse(input, Default::default()).unwrap();
    let first_tag = vdom.children()[0]
        .get(vdom.parser())
        .unwrap()
        .as_tag()
        .unwrap();

    let from_raw = first_tag.raw().try_as_utf8_str().unwrap();
    assert_eq!(from_raw, "<p>abcd</p>");
}

#[test]
fn tag_raw_abrupt_stop() {
    let input = "<p>abcd</p";

    let vdom = parse(input, Default::default()).unwrap();
    let first_tag = vdom.children()[0]
        .get(vdom.parser())
        .unwrap()
        .as_tag()
        .unwrap();

    let from_raw = first_tag.raw().try_as_utf8_str().unwrap();
    assert_eq!(from_raw, "<p>abcd</p");
}

#[test]
fn valueless_attribute_next_attribute() {
    // https://github.com/y21/tl/issues/70
    let input = r#"<button disabled id="btn">click</button>"#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    let element = dom.get_element_by_id("btn");

    assert!(element.is_some());
}

#[test]
fn valueless_attr_followed_by_valued() {
    // https://github.com/y21/tl/issues/70
    let input = r#"<input id="id" ct="I" readonly value="82.0">"#;
    let dom = parse(input, ParserOptions::default()).unwrap();
    let parser = dom.parser();

    let input_tag = dom.children()[0].get(parser).unwrap().as_tag().unwrap();
    let attrs = input_tag.attributes();

    // Check that the 'value' attribute exists (not 'alue').
    assert!(attrs.get("value").is_some(), "value attribute should exist");
    assert_eq!(attrs.get("value").unwrap().unwrap().as_utf8_str(), "82.0");

    // `readonly` should be a valueless attribute
    assert!(
        attrs.get("readonly").is_some(),
        "readonly attribute should exist"
    );
    assert!(
        attrs.get("readonly").unwrap().is_none(),
        "readonly should be valueless"
    );
}

#[test]
fn nexus_simple_api_html() {
    // Real Nexus Simple API response that triggered infinite loop
    // Key pattern: `rel="internal"  >` with double space after quoted value
    let input = r#"<!DOCTYPE html>
<html lang="en">
<head><title>Links for pyt</title>
  <meta name="api-version" value="2"/>
</head>
<body><h1>Links for pyt</h1>
        <a href="../../packages/pyt/0.5.1/pyt-0.5.1.tar.gz#sha256=965befc8e306fc38283b52d7819656ab711ad42acaa94df37ec08b824262349b" rel="internal"  >pyt-0.5.1.tar.gz</a><br/>
        <a href="../../packages/pyt/0.5.2/pyt-0.5.2.tar.gz#sha256=0e1b007ef984974f709f7c77d6a0b669bea7b0f99b016547172b10cf5ae8c525" rel="internal"  >pyt-0.5.2.tar.gz</a><br/>
        <a href="../../packages/pyt/0.5.3/pyt-0.5.3.tar.gz#sha256=c80cb7846149700f2cc8f6856d2daafbf1bb1b4597d0f7bf0b39c00f6b429732" rel="internal"  >pyt-0.5.3.tar.gz</a><br/>
        <a href="../../packages/pyt/0.6.4/pyt-0.6.4.tar.gz#sha256=49f1b87cb90cb332dbeea89370a6056117be83e43a208208f6572ab12abe2abb" rel="internal"  >pyt-0.6.4.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.1/pyt-0.7.1.tar.gz#sha256=b07e5047a8adb2177f593f976f2e8045f9dee6076f600cedbf4a1f6331ec96c4" rel="internal"  >pyt-0.7.1.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.2/pyt-0.7.2.tar.gz#sha256=0c021ffbe3c7c0e7e405fd029a0f074d473be9354d3c25cf5a3d69694c978fd7" rel="internal"  >pyt-0.7.2.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.3/pyt-0.7.3.tar.gz#sha256=9b4f22bbd716a811da65a85d5ed7b0b01989851dda6473cca9840d05766265a1" rel="internal"  >pyt-0.7.3.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.4/pyt-0.7.4.tar.gz#sha256=c153205ead215b35b839f522f06593a9bb2016aebb1157f2e8cd53d6ae05d335" rel="internal"  >pyt-0.7.4.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.5/pyt-0.7.5.tar.gz#sha256=48b28f329941271a02360d40faaccb1fb52fd4a4997995e1da80b7ed7a355b78" rel="internal"  >pyt-0.7.5.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.6/pyt-0.7.6.tar.gz#sha256=50c919236b46cf024dd6a3683b0178be8b35bc08b48650c189c04f94334fe4a5" rel="internal"  >pyt-0.7.6.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.7/pyt-0.7.7.tar.gz#sha256=1a6a5f03095d0a43d233b086422261523fd14f039846dad42dfc59e769a3a1bc" rel="internal"  >pyt-0.7.7.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.9/pyt-0.7.9.tar.gz#sha256=cc3d7e98e87ece296fb7e8c793275b35dff1c693737324b39dd51d9f08d194ab" rel="internal"  >pyt-0.7.9.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.10/pyt-0.7.10.tar.gz#sha256=7a883a24fcd425238dfaa5047d4effc7a77912d08f33111eccee27c289166aa8" rel="internal"  >pyt-0.7.10.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.11/pyt-0.7.11.tar.gz#sha256=5c78dd369f23431326933f79be85c9dfea81ee8842ab05b7f9043fb4c3d5ebe5" rel="internal"  >pyt-0.7.11.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.12/pyt-0.7.12.tar.gz#sha256=ada0f9d591b745a81fa7e96fa932cae9e98f5c9f5ce4bd9ed85992c25bfe2e18" rel="internal"  >pyt-0.7.12.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.13/pyt-0.7.13.tar.gz#sha256=8a4d1561c149aba1ede93d698a4a96c679f86deccbda5ee3596c79a0efc88d32" rel="internal"  >pyt-0.7.13.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.14/pyt-0.7.14.tar.gz#sha256=2e79b03482a1f5d5cd7df57588fe5b3d7e5504bdb0b1e3615fc66d6b52e4266e" rel="internal"  >pyt-0.7.14.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.15/pyt-0.7.15.tar.gz#sha256=629fb8bc5b34e693d7cd2008498a0c2db243b81a5b5f5cd663f58a46fa6d3301" rel="internal"  >pyt-0.7.15.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.22/pyt-0.7.22.tar.gz#sha256=40e8ab9e88c286c71ebfc8b4dc31057b4537b74faeb36d49bb0ccf9bbbd7ef66" rel="internal"  >pyt-0.7.22.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.23/pyt-0.7.23.tar.gz#sha256=73c05ffff1d185e5bc98161e115f2c3200b254a91d8fd26de874809babb02bc4" rel="internal"  >pyt-0.7.23.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.24/pyt-0.7.24.tar.gz#sha256=da5880d6001cd6a3d9a1e3af0ad501f327de6b963a478b120f3c31637ded3f1c" rel="internal"  >pyt-0.7.24.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.25/pyt-0.7.25.tar.gz#sha256=39955144f0760845003e05ede5899f667eb5af6efe99876e449cda50ee0e68d7" rel="internal"  >pyt-0.7.25.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.26/pyt-0.7.26.tar.gz#sha256=ae96638d5499c315c927b84f13a63b6961852ba85279d513270f136b93229cba" rel="internal"  >pyt-0.7.26.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.27/pyt-0.7.27.tar.gz#sha256=971c6a9e35d7757cb711cd6cef5e4f0f2aacac569252e0afa9c3eea7fa3198bf" rel="internal"  >pyt-0.7.27.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.28/pyt-0.7.28.tar.gz#sha256=e2e673ff3875deb1cb307040efdc64be120f6369a18afc391e9d7436beea281c" rel="internal"  >pyt-0.7.28.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.29/pyt-0.7.29.tar.gz#sha256=0ac0181a2847380d8af0a739d65ab5cb14cc7f135fba4c3b2613a1c7defe18de" rel="internal"  >pyt-0.7.29.tar.gz</a><br/>
        <a href="../../packages/pyt/0.7.31/pyt-0.7.31.tar.gz#sha256=592cbf3c35298e4c9da51fb18cef6b69ea61a64fe294efd96290b2451b5ae22a" rel="internal"  >pyt-0.7.31.tar.gz</a><br/>
        <a href="../../packages/pyt/1.0.5/pyt-1.0.5.tar.gz#sha256=dfb3522e17a7f3788a324fe4064b434cb5a744fae3221947af219701d1c37f75" rel="internal"  >pyt-1.0.5.tar.gz</a><br/>
        <a href="../../packages/pyt/1.3.0/pyt-1.3.0-py3-none-any.whl#sha256=3429e8cbcc066eba385d9b5df969d39491dd78a32d8ebd810b822eb91ca9c569" rel="internal" data-requires-python="&gt;=3.10" >pyt-1.3.0-py3-none-any.whl</a><br/>
        <a href="../../packages/pyt/1.3.0/pyt-1.3.0.tar.gz#sha256=acb8782effb5e7b3ef4ef07c8bc86b79e8aaf52e36ee911e521cb3efff658ee2" rel="internal" data-requires-python="&gt;=3.10" >pyt-1.3.0.tar.gz</a><br/>
        <a href="../../packages/pyt/1.4.0/pyt-1.4.0-py3-none-any.whl#sha256=67f2705a81c02039eb11f5fde898e56c2add39b7909b326db36b2fc04e547393" rel="internal" data-requires-python="&gt;=3.10" >pyt-1.4.0-py3-none-any.whl</a><br/>
        <a href="../../packages/pyt/1.4.0/pyt-1.4.0.tar.gz#sha256=09a12322f7027953b1195ea657b2b82f3c4d0b38df622bdaa6c8ff29674be216" rel="internal" data-requires-python="&gt;=3.10" >pyt-1.4.0.tar.gz</a><br/>
</body>
</html>
<script id="f5_cspm">(function(){var f5_cspm={f5_p:'<redacted>',setCharAt:function(str,index,chr){if(index>str.length-1)return str;return str.substr(0,index)+chr+str.substr(index+1);},get_byte:function(str,i){var s=(i/16)|0;i=(i&15);s=s*32;return((str.charCodeAt(i+16+s)-65)<<4)|(str.charCodeAt(i+s)-65);},set_byte:function(str,i,b){var s=(i/16)|0;i=(i&15);s=s*32;str=f5_cspm.setCharAt(str,(i+16+s),String.fromCharCode((b>>4)+65));str=f5_cspm.setCharAt(str,(i+s),String.fromCharCode((b&15)+65));return str;},set_latency:function(str,latency){latency=latency&0xffff;str=f5_cspm.set_byte(str,40,(latency>>8));str=f5_cspm.set_byte(str,41,(latency&0xff));str=f5_cspm.set_byte(str,35,2);return str;},wait_perf_data:function(){try{var wp=window.performance.timing;if(wp.loadEventEnd>0){var res=wp.loadEventEnd-wp.navigationStart;if(res<60001){var cookie_val=f5_cspm.set_latency(f5_cspm.f5_p,res);window.document.cookie='<redacted>='+encodeURIComponent(cookie_val)+';path=/;'+'';}
return;}}
catch(err){return;}
setTimeout(f5_cspm.wait_perf_data,100);return;},go:function(){var chunk=window.document.cookie.split(/\s*;\s*/);for(var i=0;i<chunk.length;++i){var pair=chunk[i].split(/\s*=\s*/);if(pair[0]=='f5_cspm'&&pair[1]=='1234')
{var d=new Date();d.setTime(d.getTime()-1000);window.document.cookie='f5_cspm=;expires='+d.toUTCString()+';path=/;'+';';setTimeout(f5_cspm.wait_perf_data,100);}}}}
f5_cspm.go();}());</script>
"#;

    let dom = parse(input, ParserOptions::default()).unwrap();
    // Verify we can find elements
    assert!(dom.children().len() > 0);
}
