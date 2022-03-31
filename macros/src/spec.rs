use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::Attribute;
use syn::Error;
use syn::Ident;
use syn::LitStr;
use syn::Result;
use syn::Token;

pub(crate) struct Spec {
    pub(crate) methods: Vec<Ident>,
    pub(crate) segments: Vec<Segment>,
}

impl Spec {
    pub(crate) fn from_attrs(name: &str, attrs: &[Attribute]) -> Result<Self> {
        attrs
            .iter()
            .find(|attr| attr.path.segments.len() == 1 && attr.path.segments[0].ident == name)
            .map(Attribute::parse_args)
            .unwrap_or_else(|| Ok(Spec::empty()))
    }

    pub(crate) fn iter_param(&self) -> impl DoubleEndedIterator<Item = &Ident> {
        self.segments.iter().filter_map(|segment| match segment {
            Segment::Lit(..) => None,
            Segment::Param(name) => Some(name),
            Segment::Wild(name) => Some(name),
        })
    }

    #[inline]
    pub(crate) fn num_param(&self) -> usize {
        self.iter_param().count()
    }
}

impl Parse for Spec {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Spec::empty());
        }

        if input.peek(LitStr) || input.peek2(syn::Token![/]) {
            return parse_segments(input).map(Spec::only_segments);
        }

        let methods = parse_methods(input)?;

        if input.is_empty() {
            return Ok(Spec::only_methods(methods));
        }

        input.parse::<syn::Token![:]>()?;

        Ok(Spec::new(methods, parse_segments(input)?))
    }
}

impl Spec {
    const fn new(methods: Vec<Ident>, segments: Vec<Segment>) -> Self {
        Spec { methods, segments }
    }

    const fn empty() -> Self {
        Spec {
            methods: Vec::new(),
            segments: Vec::new(),
        }
    }

    const fn only_methods(methods: Vec<Ident>) -> Self {
        Spec {
            methods,
            segments: Vec::new(),
        }
    }

    const fn only_segments(segments: Vec<Segment>) -> Self {
        Spec {
            segments,
            methods: Vec::new(),
        }
    }
}

fn parse_methods(input: ParseStream) -> Result<Vec<Ident>> {
    let mut methods = vec![input.parse()?];

    while input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
        methods.push(input.parse()?);
    }

    Ok(methods)
}

fn parse_segments(input: ParseStream) -> Result<Vec<Segment>> {
    let mut segments = vec![input.parse()?];

    while input.peek(Token![/]) {
        input.parse::<Token![/]>()?;
        let segment = input.parse()?;

        if matches!(segment, Segment::Wild(..)) && !input.is_empty() {
            return Err(Error::new(
                input.span(),
                "wildcard (*) must be the last segment",
            ));
        }

        segments.push(segment);
    }

    Ok(segments)
}

pub(crate) enum Segment {
    Lit(LitStr),
    Param(Ident),
    Wild(Ident),
}

impl Parse for Segment {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(LitStr) {
            return input.parse().map(Self::Lit);
        }

        let ident = input.parse()?;
        if !input.peek(Token![*]) {
            return Ok(Segment::Param(ident));
        }

        input.parse::<Token![*]>().map(|_| Segment::Wild(ident))
    }
}

#[test]
#[cfg(test)]
fn segment() {
    let stream = syn::parse_quote!("path");
    let segment = syn::parse2::<Segment>(stream).unwrap();
    assert!(matches!(segment, Segment::Lit(..)));

    let stream = syn::parse_quote!(param);
    let segment = syn::parse2::<Segment>(stream).unwrap();
    assert!(matches!(segment, Segment::Param(..)));

    let stream = syn::parse_quote!(wild*);
    let segment = syn::parse2::<Segment>(stream).unwrap();
    assert!(matches!(segment, Segment::Wild(..)));
}

#[test]
#[cfg(test)]
fn only_method() {
    let stream = syn::parse_quote!(GET);
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_ok());

    let stream = syn::parse_quote!(GET,);
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_err());

    let stream = syn::parse_quote!(GET, POST);
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_ok());

    let stream = syn::parse_quote!(GET, POST:);
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_err());
}

#[test]
#[cfg(test)]
fn only_path() {
    let stream = syn::parse_quote!("entity");
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_ok());

    let stream = syn::parse_quote!("entity" /);
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_err());

    let stream = syn::parse_quote!("entity" / param);
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_ok());

    let stream = syn::parse_quote!("entity" / param /);
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_err());

    let stream = syn::parse_quote!("entity" / param / *);
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_err());

    let stream = syn::parse_quote!("entity" / param / wild*);
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_ok());

    let stream = syn::parse_quote!("entity" / param / wild* /);
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_err());

    let stream = syn::parse_quote!("entity" / param / wild* / err);
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_err());
}

#[test]
#[cfg(test)]
fn method_and_path() {
    let stream = syn::parse_quote!(GET: "path");
    let segment = syn::parse2::<Spec>(stream);
    assert!(segment.is_ok());
}
