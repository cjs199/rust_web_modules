use syn::{parse::Parse, Token};
use syn::{parse::ParseStream, LitStr};
extern crate proc_macro;

pub struct RedisMqModel {
    pub que: LitStr,
    pub group: LitStr,
}

mod kw {
    use syn::custom_keyword;
    custom_keyword!(que);
    custom_keyword!(group);
}

impl Parse for RedisMqModel {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let (mut que, mut group) = (None, None);
        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::que) {
                input.parse::<kw::que>()?;
                input.parse::<Token![=]>()?;
                que = Some(input.parse::<LitStr>()?);
            } else if lookahead.peek(kw::group) {
                input.parse::<kw::group>()?;
                input.parse::<Token![=]>()?;
                group = Some(input.parse::<LitStr>()?);
            } else {
                return Err(input.error("无效参数"));
            }
            if let Err(_) = input.parse::<Token![,]>() {
                break;
            }
        }
        match (que, group) {
            (Some(que), Some(group)) => Ok(Self { que, group }),
            _ => Err(input.error("缺失必须参数que 或 group")),
        }
    }
}
