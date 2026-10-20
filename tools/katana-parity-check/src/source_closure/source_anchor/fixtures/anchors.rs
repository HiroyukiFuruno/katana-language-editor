enum Mode {
    開く,
}

struct Entry {
    value: usize,
}

#[test]
fn test_anchor() {
    let _emoji = "日本語✈️";
    let _entry = Entry { value: 2 };
    let _items = vec![Entry { value: 3 }; 2];
    unknown_macro!(callback);
}

fn ordinary() {}
