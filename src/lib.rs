use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};
use std::iter::Peekable;

#[proc_macro_attribute]
pub fn pub_if(params: TokenStream, item: TokenStream) -> TokenStream {
    let cfg_condition = parse_cfg_condition(params);

    let mut parser = Parser::new(item.clone());
    let struct_def = parser.parse_struct();

    let cfg_enabled_version = generate_struct_with_cfg(&struct_def, &cfg_condition, true);
    let cfg_disabled_version = generate_struct_with_cfg(&struct_def, &cfg_condition, false);

    let mut output = TokenStream::new();
    output.extend(cfg_enabled_version);
    output.extend(cfg_disabled_version);
    output
}

fn parse_cfg_condition(attr: TokenStream) -> Vec<TokenTree> {
    attr.into_iter().collect()
}

struct Parser {
    tokens: Peekable<std::vec::IntoIter<TokenTree>>,
}

impl Parser {
    fn new(stream: TokenStream) -> Self {
        Self {
            tokens: stream
                .into_iter()
                .collect::<Vec<_>>()
                .into_iter()
                .peekable(),
        }
    }

    fn parse_struct(&mut self) -> StructDef {
        let mut other_attributes = Vec::new();
        let mut visibility = Vec::new();

        while let Some(token) = self.tokens.peek() {
            match token {
                TokenTree::Punct(p) if p.as_char() == '#' => {
                    other_attributes.push(self.parse_attribute());
                }
                TokenTree::Ident(ident) if ident.to_string() == "pub" => {
                    visibility.push(self.tokens.next().unwrap());
                    // consume Group if exists (e.g., pub(crate))
                    if let Some(TokenTree::Group(_)) = self.tokens.peek() {
                        visibility.push(self.tokens.next().unwrap());
                    }
                    break;
                }
                TokenTree::Ident(ident) if ident.to_string() == "struct" => {
                    break;
                }
                _ => {
                    self.tokens.next();
                }
            }
        }

        let struct_token = self.tokens.next().unwrap();
        let name = self.tokens.next().unwrap();

        let generics = if matches!(self.tokens.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '<')
        {
            Some(self.parse_generics())
        } else {
            None
        };

        let fields_group = self.tokens.next().unwrap();

        StructDef {
            attributes: other_attributes,
            visibility,
            struct_token,
            name,
            generics,
            fields_group,
        }
    }

    fn parse_attribute(&mut self) -> Vec<TokenTree> {
        let mut attr = Vec::new();
        attr.push(self.tokens.next().unwrap()); // #

        if let Some(TokenTree::Group(_)) = self.tokens.peek() {
            attr.push(self.tokens.next().unwrap());
        }

        attr
    }

    fn parse_generics(&mut self) -> Vec<TokenTree> {
        let mut generics = Vec::new();
        let mut depth = 0;

        while let Some(token) = self.tokens.peek() {
            match token {
                TokenTree::Punct(p) if p.as_char() == '<' => {
                    depth += 1;
                    generics.push(self.tokens.next().unwrap());
                }
                TokenTree::Punct(p) if p.as_char() == '>' => {
                    generics.push(self.tokens.next().unwrap());
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {
                    generics.push(self.tokens.next().unwrap());
                }
            }
        }

        generics
    }
}

struct StructDef {
    attributes: Vec<Vec<TokenTree>>,
    visibility: Vec<TokenTree>,
    struct_token: TokenTree,
    name: TokenTree,
    generics: Option<Vec<TokenTree>>,
    // We could parse only this group `{..}` but
    // generics `<..>` is represented not as a group but as a sequence of tokens
    // and can contain groups of curly braces inside (as const init).
    fields_group: TokenTree,
}

fn generate_struct_with_cfg(
    struct_def: &StructDef,
    cfg_condition: &[TokenTree],
    make_pub: bool,
) -> TokenStream {
    let mut output = TokenStream::new();

    let cfg_attr = if make_pub {
        create_cfg_attribute(cfg_condition, false)
    } else {
        create_cfg_attribute(cfg_condition, true)
    };
    output.extend(cfg_attr);

    for attr in &struct_def.attributes {
        output.extend(attr.iter().cloned().collect::<TokenStream>());
    }

    output.extend(
        struct_def
            .visibility
            .iter()
            .cloned()
            .collect::<TokenStream>(),
    );
    output.extend(std::iter::once(struct_def.struct_token.clone()));
    output.extend(std::iter::once(struct_def.name.clone()));

    if let Some(generics) = &struct_def.generics {
        output.extend(generics.iter().cloned().collect::<TokenStream>());
    }

    if let TokenTree::Group(g) = &struct_def.fields_group {
        let new_fields = if make_pub {
            make_fields_public(g)
        } else {
            struct_def.fields_group.clone()
        };
        output.extend(std::iter::once(new_fields));
    }

    output
}

fn create_cfg_attribute(condition: &[TokenTree], negate: bool) -> TokenStream {
    let mut output = TokenStream::new();

    output.extend(std::iter::once(TokenTree::Punct(Punct::new(
        '#',
        Spacing::Alone,
    ))));

    let mut cfg_tokens = TokenStream::new();
    cfg_tokens.extend(std::iter::once(TokenTree::Ident(Ident::new(
        "cfg",
        Span::call_site(),
    ))));

    let mut inner_tokens = TokenStream::new();
    if negate {
        inner_tokens.extend(std::iter::once(TokenTree::Ident(Ident::new(
            "not",
            Span::call_site(),
        ))));

        let condition_group = TokenTree::Group(Group::new(
            Delimiter::Parenthesis,
            condition.iter().cloned().collect(),
        ));
        inner_tokens.extend(std::iter::once(condition_group));
    } else {
        inner_tokens.extend(condition.iter().cloned().collect::<TokenStream>());
    }

    cfg_tokens.extend(std::iter::once(TokenTree::Group(Group::new(
        Delimiter::Parenthesis,
        inner_tokens,
    ))));

    output.extend(std::iter::once(TokenTree::Group(Group::new(
        Delimiter::Bracket,
        cfg_tokens,
    ))));

    output
}

fn make_fields_public(group: &Group) -> TokenTree {
    let mut new_fields = TokenStream::new();
    let tokens: Vec<TokenTree> = group.stream().into_iter().collect();
    let mut i = 0;
    let mut just_saw_pub = false;

    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Ident(ident) => {
                let ident_str = ident.to_string();

                if ident_str == "pub" {
                    new_fields.extend(std::iter::once(tokens[i].clone()));
                    just_saw_pub = true;
                } else if !just_saw_pub
                    && i + 1 < tokens.len()
                    && matches!(&tokens[i + 1], TokenTree::Punct(p) if p.as_char() == ':')
                {
                    new_fields.extend(std::iter::once(TokenTree::Ident(Ident::new(
                        "pub",
                        Span::call_site(),
                    ))));
                    new_fields.extend(std::iter::once(tokens[i].clone()));
                    just_saw_pub = false;
                } else {
                    new_fields.extend(std::iter::once(tokens[i].clone()));
                    just_saw_pub = false;
                }
            }
            TokenTree::Punct(p) if p.as_char() == ',' || p.as_char() == ';' => {
                new_fields.extend(std::iter::once(tokens[i].clone()));
                just_saw_pub = false;
            }
            _ => {
                new_fields.extend(std::iter::once(tokens[i].clone()));
            }
        }
        i += 1;
    }

    TokenTree::Group(Group::new(group.delimiter(), new_fields))
}
