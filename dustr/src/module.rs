use ::darling::FromDeriveInput;

impl crate::Module {
    /// Generate the `Module` (and all submodules) from the provided crate.
    pub fn from_crate(path: ::std::path::PathBuf) -> ::anyhow::Result<Self> {
        let manifest = ::cargo_toml::Manifest::from_path(&path.join("Cargo.toml"))?;
        let name = manifest.package.ok_or_else(|| {
            ::anyhow::Error::msg(format!("empty [package] section for {}", path.display()))
        })?.name.to_lowercase();
        Self::from_file(name.clone(), name, path.join("src/lib.rs"))
    }

    /// Generate the `Module` (and all submodules) from the provided file.
    pub fn from_file(
        name: String,
        crate_name: String,
        path: ::std::path::PathBuf
    ) -> ::anyhow::Result<Self> {
        Self::from_items(
            name,
            crate_name,
            path.clone(),
            ::syn::parse_file(&::std::fs::read_to_string(path)?)?.items,
        )
    }

    pub fn from_items(
        name: String,
        crate_name: String,
        path: ::std::path::PathBuf,
        items: Vec<::syn::Item>,
    ) -> ::anyhow::Result<Self> {
        let mut structs = vec![];
        let mut enums = vec![];
        let mut functions = vec![];
        let mut subs = vec![];

        for item in items {
            if let ::syn::Item::Mod(im) = item {
                let module = Self::from_itemmod(crate_name.clone(), path.clone(), im)?;
                if !module.is_empty() {
                    subs.push(module);
                }
            } else if let ::syn::Item::Struct(is) = item {
                if let Some(data) = filter_item_struct(is)? {
                    structs.push(data);
                }
            } else if let ::syn::Item::Enum(ie) = item {
                if let Some(data) = filter_item_enum(ie)? {
                    enums.push(data);
                }
            } else if let ::syn::Item::Fn(ifn) = item {
                if let Some(func) = filter_item_fn(ifn) {
                    functions.push(func);
                }
            }
        }

        Ok(Self{name, crate_name, structs, enums, functions, subs})
    }

    pub fn from_itemmod(
        crate_name: String,
        parent: ::std::path::PathBuf,
        im: ::syn::ItemMod,
    ) -> ::anyhow::Result<Self> {
        let name = im.ident.to_string();

        if let Some((_, items)) = im.content {
            Self::from_items(name, crate_name, parent, items)
        } else {
            let parent = parent.parent().ok_or_else(|| {
                ::anyhow::Error::msg(format!("cannot get parent of {}", parent.display()))
            })?;

            // TODO: not handled is when a submodule has name.rs and name/ for its submodules
            // we only handle the name/mod.rs for submodules with sub-submodule files.
            let mut file = parent.join(format!("{}.rs", name));
            if !file.exists() {
                file = parent.join(format!("{}/mod.rs", name));
            }

            Self::from_file(name, crate_name, file)
        }
    }

    fn is_empty(&self) -> bool {
        self.structs.is_empty() &&
            self.enums.is_empty() &&
            self.functions.is_empty() &&
            self.subs.is_empty()
    }
}

fn filter_item_struct(is: ::syn::ItemStruct) -> ::anyhow::Result<Option<::ffishim::Data>> {
    if derives_ffishim(&is.attrs) {
        let derive_input = ::syn::DeriveInput{
            attrs: is.attrs.clone(),
            vis: is.vis.clone(),
            ident: is.ident.clone(),
            generics: is.generics.clone(),
            data: ::syn::Data::Struct(::syn::DataStruct{
                struct_token: is.struct_token,
                fields: is.fields.clone(),
                semi_token: is.semi_token,
            }),
        };

        Ok(Some(::ffishim::Data::from_derive_input(&derive_input).map_err(|e| {
            ::anyhow::Error::msg(format!("{}", e)) // needed because darling error is not sync
        })?))
    } else {
        Ok(None)
    }
}

fn filter_item_enum(ie: ::syn::ItemEnum) -> ::anyhow::Result<Option<::ffishim::Data>> {
    if derives_ffishim(&ie.attrs) {
        let derive_input = ::syn::DeriveInput{
            attrs: ie.attrs.clone(),
            vis: ie.vis.clone(),
            ident: ie.ident.clone(),
            generics: ie.generics.clone(),
            data: ::syn::Data::Enum(::syn::DataEnum{
                enum_token: ie.enum_token,
                brace_token: ie.brace_token,
                variants: ie.variants.clone(),
            }),
        };

        Ok(Some(::ffishim::Data::from_derive_input(&derive_input).map_err(|e| {
            ::anyhow::Error::msg(format!("{}", e)) // needed because darling error is not sync
        })?))
    } else {
        Ok(None)
    }
}

fn filter_item_fn(ifn: ::syn::ItemFn) -> Option<::syn::ItemFn> {
    if derives_ffishim_function(&ifn.attrs) {
        Some(ifn)
    } else {
        None
    }
}

fn derives_ffishim(attrs: &Vec<::syn::Attribute>) -> bool {
    attrs.iter().any(|attr| if let Ok(::syn::Meta::List(list)) = attr.parse_meta() {
        if list.path.is_ident("derive") {
            list.nested.iter().any(|meta| if let ::syn::NestedMeta::Meta(m) = meta {
                if let ::syn::Meta::Path(p) = m {
                    p.is_ident("FFIShim")
                } else {
                    false
                }
            } else {
                false
            })
        } else {
            false
        }
    } else {
        false
    })
}

fn derives_ffishim_function(attrs: &Vec<::syn::Attribute>) -> bool {
    attrs.iter().any(|attr| attr.path.is_ident("ffishim_function"))
}
