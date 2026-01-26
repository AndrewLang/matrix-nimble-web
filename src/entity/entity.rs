pub trait Entity: Send + Sync + 'static {
    type Id: Send + Sync + Clone + 'static;

    fn id(&self) -> &Self::Id;

    fn name() -> &'static str;

    fn plural_name() -> String {
        let name = Self::name();
        let lower = name.to_ascii_lowercase();
        let bytes = lower.as_bytes();
        let ends_with = |suffix: &str| lower.ends_with(suffix);
        if ends_with("s") || ends_with("x") || ends_with("z") || ends_with("ch") || ends_with("sh")
        {
            return format!("{name}es");
        }
        if bytes.len() >= 2 && bytes[bytes.len() - 1] == b'y' {
            let prev = bytes[bytes.len() - 2];
            let is_vowel = matches!(prev, b'a' | b'e' | b'i' | b'o' | b'u');
            if !is_vowel {
                let mut base = name.to_string();
                base.pop();
                return format!("{base}ies");
            }
        }
        format!("{name}s")
    }
}
