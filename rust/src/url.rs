// URL 编解码：对齐 JS encodeURIComponent / decodeURIComponent。
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};

/// encodeURIComponent 不转义 - _ . ! ~ * ' ( )
pub const URI_COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'!')
    .remove(b'~')
    .remove(b'*')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')');

pub fn encode(value: &str) -> String {
    percent_encoding::utf8_percent_encode(value, URI_COMPONENT).to_string()
}

pub fn decode(value: &str) -> String {
    percent_encoding::percent_decode_str(value)
        .decode_utf8_lossy()
        .to_string()
}
