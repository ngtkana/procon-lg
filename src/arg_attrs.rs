use proc_macro2::TokenStream;
use syn::{Attribute, Expr, Result};

/// 引数の属性情報を表現する構造体
#[derive(Clone)]
pub enum ArgAttribute {
    /// #[no_debug] - デバッグ出力から除外
    NoDebug,
    /// #[fmt(closure)] - カスタムフォーマッタを使用
    Fmt { formatter: Expr },
}

/// 引数に適用された属性の解析結果
pub struct ArgAttributes {
    pub attrs: Vec<ArgAttribute>,
}

impl ArgAttributes {
    /// 属性リストから ArgAttributes を作成
    pub fn from_attrs(attrs: &[Attribute]) -> Result<Self> {
        let mut parsed_attrs = Vec::new();

        for attr in attrs {
            if attr.path().is_ident("no_debug") {
                parsed_attrs.push(ArgAttribute::NoDebug);
            } else if attr.path().is_ident("fmt") {
                let formatter = attr.parse_args::<Expr>()?;
                parsed_attrs.push(ArgAttribute::Fmt { formatter });
            }
        }

        Ok(ArgAttributes {
            attrs: parsed_attrs,
        })
    }

    /// この引数をデバッグ出力に含めるかどうか
    pub fn should_include_in_debug(&self) -> bool {
        !self
            .attrs
            .iter()
            .any(|attr| matches!(attr, ArgAttribute::NoDebug))
    }

    /// カスタムフォーマッタを取得（最初に見つかったもの）
    pub fn get_custom_formatter(&self) -> Option<&Expr> {
        self.attrs.iter().find_map(|attr| {
            if let ArgAttribute::Fmt { formatter } = attr {
                Some(formatter)
            } else {
                None
            }
        })
    }

    /// フォーマット用のトークンストリームを生成
    pub fn generate_format_tokens(&self, arg_name: &syn::Ident) -> TokenStream {
        if let Some(formatter) = self.get_custom_formatter() {
            quote::quote! {
                (#formatter)(&#arg_name)
            }
        } else {
            quote::quote! {
                format!("{:?}", #arg_name)
            }
        }
    }
}
