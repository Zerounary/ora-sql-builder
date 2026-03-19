pub fn sanitize_filter_value(value: &str) -> String {
    let mut sanitized = String::new();
    let chars: Vec<char> = value.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        if chars[index] == '%'
            && index + 2 < chars.len()
            && chars[index + 1].is_ascii_hexdigit()
            && chars[index + 2].is_ascii_hexdigit()
        {
            index += 3;
            continue;
        }

        sanitized.push(chars[index]);
        index += 1;
    }

    sanitized
}

pub fn ordinal_index_of(dbname: &str, arg: char, x: usize) -> usize {
    let mut index = 0;
    let mut count = 0;
    for (i, ch) in dbname.char_indices() {
        if ch == arg {
            count += 1;
            if count == x {
                index = i;
                break;
            }
        }
    }
    index
}

pub fn push_unique(list: &mut Vec<String>, item: String) {
    if !list.contains(&item) {
        list.push(item.to_string());
    }
}
