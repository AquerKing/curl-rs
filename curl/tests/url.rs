use curl_derive::FromRegex;

#[derive(FromRegex, Debug, PartialEq, Eq)]
#[regex(r"^(?P<scheme>[a-zA-Z][a-zA-Z0-9+.\-]*)://(?P<host>[^/:?#]+)(?::(?P<port>\d+))?(?P<path>/[^?#]*)?(?:\?(?P<query>[^#]*))?(?:#(?P<fragment>.*))?$")]
struct Url {
    scheme: String,
    host: String,
    port: Option<u16>,
    path: Option<String>,
    query: Option<String>,
    fragment: Option<String>,
}

#[test]
fn full_url() {
    let u: Url = "https://example.com:8080/path/to?x=1#frag".parse().unwrap();
    assert_eq!(u.scheme, "https");
    assert_eq!(u.host, "example.com");
    assert_eq!(u.port, Some(8080));
    assert_eq!(u.path.as_deref(), Some("/path/to"));
    assert_eq!(u.query.as_deref(), Some("x=1"));
    assert_eq!(u.fragment.as_deref(), Some("frag"));
}

#[test]
fn minimal_url() {
    let u: Url = "http://localhost".parse().unwrap();
    assert_eq!(u.scheme, "http");
    assert_eq!(u.host, "localhost");
    assert_eq!(u.port, None);
    assert_eq!(u.path, None);
    assert_eq!(u.query, None);
    assert_eq!(u.fragment, None);
}

#[test]
fn no_match() {
    let r = "not a url".parse::<Url>();
    assert!(matches!(r, Err(curl_rs::Error::NoMatch)));
}
