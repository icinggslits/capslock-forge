#[allow(unused)]





pub mod string {
    /// 字符串操作
    /// 
    /// 去除头尾单次匹配的子字符串
    pub trait TrimCharMatches {
        /// 去除头部单次匹配的子字符串
        /// 
        /// # 例子
        /// 
        /// ```
        /// use units::string::TrimCharMatches;
        /// let text = "--123--".to_string();
        /// assert_eq!(text.trim_start_char_matches("-"), "-123--");
        /// ```
        fn trim_start_char_matches<'a, P: AsRef<str>>(&'a self, pat: P) -> &'a str;
    
        /// 去除尾部单次匹配的子字符串
        /// 
        /// # 例子
        /// 
        /// ```
        /// use units::string::TrimCharMatches;
        /// let text = "--123--".to_string();
        /// assert_eq!(text.trim_end_char_matches("-"), "--123-");
        /// ```
        fn trim_end_char_matches<'a, P: AsRef<str>>(&'a self, pat: P) -> &'a str;
    
        /// 去除头尾单次匹配的子字符串
        /// 
        /// # 例子
        /// 
        /// ```
        /// use units::string::TrimCharMatches;
        /// let text = "--123--".to_string();
        /// assert_eq!(text.trim_char_matches("-"), "-123-");
        /// ```
        fn trim_char_matches<'a, P: AsRef<str>>(&'a self, pat: P) -> &'a str;
    }
    
    impl TrimCharMatches for String {
        fn trim_start_char_matches<'a, P: AsRef<str>>(&'a self, pat: P) -> &'a str {
            trim_start_char_matches(self, pat)
        }
    
        fn trim_end_char_matches<'a, P: AsRef<str>>(&'a self, pat: P) -> &'a str {
            trim_end_char_matches(self, pat)
        }
    
        fn trim_char_matches<'a, P: AsRef<str>>(&'a self, pat: P) -> &'a str {
            let s = trim_start_char_matches(self, &pat);
            trim_end_char_matches(s, &pat)
        }
    }
    
    impl TrimCharMatches for &str {
        fn trim_start_char_matches<'a, P: AsRef<str>>(&'a self, pat: P) -> &'a str {
            trim_start_char_matches(self, pat)
        }
    
        fn trim_end_char_matches<'a, P: AsRef<str>>(&'a self, pat: P) -> &'a str {
            trim_end_char_matches(self, pat)
        }
    
        fn trim_char_matches<'a, P: AsRef<str>>(&'a self, pat: P) -> &'a str {
            let s = trim_start_char_matches(self, &pat);
            trim_end_char_matches(s, &pat)
        }
    }
    
    fn trim_start_char_matches<'a, P: AsRef<str>>(that: &'a str, pat: P) -> &'a str {
        let pat = pat.as_ref();
        let pat_len = pat.len();
        if that[0..pat_len] == *pat {
            return &that[pat_len..]
        }
        return that
    }
    
    fn trim_end_char_matches<'a, P: AsRef<str>>(that: &'a str, pat: P) -> &'a str {
        let pat = pat.as_ref();
        let pat_len = pat.len();
        let len = that.len();
        let p = len - pat_len;
        if that[p..len] == *pat {
            return &that[..p]
        }
        return that
    }
    
}

pub mod file_io {
    use std::{fs, io, path::Path};

    /// 创建并写入文件
    /// 
    /// `std::fs::write`的包装，目标路径没有相应的文件夹也会正确创建文件
    /// 
    /// 与`std::fs::write`相同，覆盖写入
    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, contents)
    }
}
