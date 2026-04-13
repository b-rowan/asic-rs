use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Attribute, Data, DeriveInput, Fields, LitStr, Type, parse_macro_input};

// ── attribute parsing ──────────────────────────────────────────────────────

/// Options parsed from `#[pydantic(...)]` on a *field*.
#[derive(Default)]
struct FieldOpts {
    /// `serialize_with = "to_string"` – serialize via `.to_string()`.
    /// `serialize_with = "nested"` – serialize via `._pydantic_serialize(py)`.
    serialize_with: Option<String>,
    /// `validate_with = "from_str"` – try `extract::<T>()`, then parse from str.
    /// `validate_with = "nested"` – try `extract::<T>()`, then call `T::_pydantic_validate`.
    validate_with: Option<String>,
}

/// Options parsed from `#[pydantic(...)]` on the *type* (struct or enum).
#[derive(Default)]
struct TypeOpts {
    /// `validate = "from_str"` (enums) – try `extract::<Self>()`, then parse.
    validate: Option<String>,
    /// `serialize = "to_string"` (enums) – serialise via `.to_string()`.
    serialize: Option<String>,
}

fn parse_pydantic_field_opts(attrs: &[Attribute]) -> FieldOpts {
    let mut opts = FieldOpts::default();
    for attr in attrs {
        if !attr.path().is_ident("pydantic") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("serialize_with") {
                let value: LitStr = meta.value()?.parse()?;
                opts.serialize_with = Some(value.value());
            } else if meta.path.is_ident("validate_with") {
                let value: LitStr = meta.value()?.parse()?;
                opts.validate_with = Some(value.value());
            }
            Ok(())
        });
    }
    opts
}

fn parse_pydantic_type_opts(attrs: &[Attribute]) -> TypeOpts {
    let mut opts = TypeOpts::default();
    for attr in attrs {
        if !attr.path().is_ident("pydantic") {
            continue;
        }
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("validate") {
                let value: LitStr = meta.value()?.parse()?;
                opts.validate = Some(value.value());
            } else if meta.path.is_ident("serialize") {
                let value: LitStr = meta.value()?.parse()?;
                opts.serialize = Some(value.value());
            }
            Ok(())
        });
    }
    opts
}

// ── type introspection helpers ─────────────────────────────────────────────

/// If `ty` is `Option<T>`, returns `Some(T)`.
fn option_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Option" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    let syn::GenericArgument::Type(inner) = ab.args.first()? else {
        return None;
    };
    Some(inner)
}

/// If `ty` is `Vec<T>`, returns `Some(T)`.
fn vec_inner(ty: &Type) -> Option<&Type> {
    let Type::Path(tp) = ty else { return None };
    let seg = tp.path.segments.last()?;
    if seg.ident != "Vec" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(ab) = &seg.arguments else {
        return None;
    };
    let syn::GenericArgument::Type(inner) = ab.args.first()? else {
        return None;
    };
    Some(inner)
}

// ── code generation helpers ────────────────────────────────────────────────

/// The identical `__get_pydantic_core_schema__` body shared by all types.
fn schema_method() -> TokenStream2 {
    quote! {
        #[classmethod]
        fn __get_pydantic_core_schema__<'py>(
            cls: &::pyo3::Bound<'py, ::pyo3::types::PyType>,
            _source_type: &::pyo3::Bound<'py, ::pyo3::PyAny>,
            _handler: &::pyo3::Bound<'py, ::pyo3::PyAny>,
        ) -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::PyAny>> {
            let py = cls.py();
            let cs = py.import("pydantic_core.core_schema")?;

            let validate_fn = cls.getattr("_pydantic_validate")?;
            let serialize_fn = cls.getattr("_pydantic_serialize")?;

            let ser_kwargs = ::pyo3::types::PyDict::new(py);
            ser_kwargs.set_item("info_arg", false)?;
            let serializer = cs.call_method(
                "plain_serializer_function_ser_schema",
                (&serialize_fn,),
                Some(&ser_kwargs),
            )?;

            let val_kwargs = ::pyo3::types::PyDict::new(py);
            val_kwargs.set_item("serialization", &serializer)?;
            cs.call_method(
                "no_info_plain_validator_function",
                (&validate_fn,),
                Some(&val_kwargs),
            )
        }
    }
}

/// `model_validate(cls, obj)` — mirrors `BaseModel.model_validate`.
/// Delegates directly to `_pydantic_validate` (same impl block, no Python dispatch).
fn model_validate_method() -> TokenStream2 {
    quote! {
        /// Validate and construct `Self` from a Python object or dict.
        /// Equivalent to `pydantic.TypeAdapter(Self).validate_python(obj)`.
        #[classmethod]
        fn model_validate(
            cls: &::pyo3::Bound<'_, ::pyo3::types::PyType>,
            obj: &::pyo3::Bound<'_, ::pyo3::PyAny>,
        ) -> ::pyo3::PyResult<Self> {
            Self::_pydantic_validate(cls, obj)
        }
    }
}

/// `model_dump(*, mode=None)` for *structs* — returns the serialised `dict`.
fn model_dump_struct_method() -> TokenStream2 {
    quote! {
        /// Serialise to a Python `dict`.
        /// Equivalent to `pydantic.TypeAdapter(Self).dump_python(self)`.
        #[pyo3(signature = (*, mode = None))]
        fn model_dump<'py>(
            &self,
            py: ::pyo3::Python<'py>,
            mode: Option<&str>,
        ) -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::PyAny>> {
            let _ = mode;
            Ok(self._pydantic_serialize(py)?.into_any())
        }
    }
}

/// `model_dump(*, mode=None)` for *enums* — returns the serialised `str`.
fn model_dump_enum_method() -> TokenStream2 {
    quote! {
        /// Serialise to a Python `str` (the display representation).
        /// Equivalent to `pydantic.TypeAdapter(Self).dump_python(self)`.
        #[pyo3(signature = (*, mode = None))]
        fn model_dump<'py>(
            &self,
            py: ::pyo3::Python<'py>,
            mode: Option<&str>,
        ) -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::PyAny>> {
            let _ = mode;
            use ::pyo3::IntoPyObject as _;
            self._pydantic_serialize()
                .into_pyobject(py)
                .map(|b| b.into_any())
                .map_err(Into::into)
        }
    }
}

/// Generates the `extract-or-parse-from-str` snippet for a single value object.
/// Used when `validate_with = "from_str"` is set on a field or the whole type.
fn extract_or_from_str(ty: &TokenStream2, obj: &TokenStream2) -> TokenStream2 {
    quote! {
        #obj.extract::<#ty>().or_else(|_| {
            #obj.extract::<String>()
                .and_then(|s| s.parse::<#ty>()
                    .map_err(|e| ::pyo3::exceptions::PyValueError::new_err(e.to_string())))
        })?
    }
}

/// Generates code to safely retrieve a dict item — returns `None` if key is absent.
fn safe_get_item(field_str: &str) -> TokenStream2 {
    quote! {
        {
            let __raw = value.get_item(#field_str);
            match __raw {
                Ok(v) if !v.is_none() => Some(v),
                _ => None,
            }
        }
    }
}

// ── validation helpers for "nested" ───────────────────────────────────────

/// `validate_with = "nested"` for a field of type `T`, `Option<T>`, or `Vec<T>`.
///
/// - `T`         → `extract::<T>()` or fall back to `T::_pydantic_validate`
/// - `Option<T>` → missing / None key yields `None`; otherwise as above
/// - `Vec<T>`    → extract each element as above
fn nested_validate_expr(field_str: &str, ty: &Type) -> TokenStream2 {
    if let Some(inner) = option_inner(ty) {
        let get = safe_get_item(field_str);
        quote! {
            match #get {
                None => None,
                Some(__item) => {
                    let __py = __item.py();
                    Some(__item.extract::<#inner>().or_else(|_| {
                        let __cls = __py.get_type::<#inner>();
                        #inner::_pydantic_validate(&__cls, &__item)
                    })?)
                }
            }
        }
    } else if let Some(inner) = vec_inner(ty) {
        quote! {
            {
                let __list = value.get_item(#field_str)?
                    .extract::<Vec<::pyo3::Bound<'_, ::pyo3::PyAny>>>()?;
                let mut __result = Vec::with_capacity(__list.len());
                for __item in __list {
                    let __py = __item.py();
                    let __val = __item.extract::<#inner>().or_else(|_| {
                        let __cls = __py.get_type::<#inner>();
                        #inner::_pydantic_validate(&__cls, &__item)
                    })?;
                    __result.push(__val);
                }
                __result
            }
        }
    } else {
        quote! {
            {
                let __item = value.get_item(#field_str)?;
                let __py = __item.py();
                __item.extract::<#ty>().or_else(|_| {
                    let __cls = __py.get_type::<#ty>();
                    #ty::_pydantic_validate(&__cls, &__item)
                })?
            }
        }
    }
}

// ── serialization helpers for "nested" ────────────────────────────────────

/// `serialize_with = "nested"` for a field of type `T`, `Option<T>`, or `Vec<T>`.
fn nested_serialize_stmt(field_name: &syn::Ident, field_str: &str, ty: &Type) -> TokenStream2 {
    if option_inner(ty).is_some() {
        quote! {
            dict.set_item(
                #field_str,
                self.#field_name.as_ref()
                    .map(|__v| __v._pydantic_serialize(py))
                    .transpose()?,
            )?;
        }
    } else if vec_inner(ty).is_some() {
        quote! {
            dict.set_item(
                #field_str,
                self.#field_name.iter()
                    .map(|__v| __v._pydantic_serialize(py))
                    .collect::<::pyo3::PyResult<Vec<_>>>()?,
            )?;
        }
    } else {
        quote! {
            dict.set_item(#field_str, self.#field_name._pydantic_serialize(py)?)?;
        }
    }
}

// ── default validation for undecorated fields ─────────────────────────────

/// Default extraction for a field with no `validate_with`.
/// Handles `Option<T>` (safe key lookup) and `Vec<T>` / `T` (direct get_item).
fn default_validate_expr(field_str: &str, ty: &Type) -> TokenStream2 {
    if option_inner(ty).is_some() {
        let get = safe_get_item(field_str);
        quote! {
            match #get {
                None => None,
                Some(__item) => Some(__item.extract()?),
            }
        }
    } else {
        quote! { value.get_item(#field_str)?.extract::<#ty>()? }
    }
}

// ── struct generation ──────────────────────────────────────────────────────

fn generate_struct_impl(name: &syn::Ident, data: &syn::DataStruct) -> TokenStream2 {
    let fields = match &data.fields {
        Fields::Named(f) => &f.named,
        _ => panic!("PydanticCompat requires named fields"),
    };

    let validate_fields: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let field_str = field_name.to_string();
            let ty = &f.ty;
            let opts = parse_pydantic_field_opts(&f.attrs);

            let extraction = match opts.validate_with.as_deref() {
                Some("from_str") => {
                    let ty_ts = quote! { #ty };
                    let obj = quote! { value.get_item(#field_str)? };
                    extract_or_from_str(&ty_ts, &obj)
                }
                Some("nested") => nested_validate_expr(&field_str, ty),
                _ => default_validate_expr(&field_str, ty),
            };

            quote! { #field_name: #extraction, }
        })
        .collect();

    let serialize_fields: Vec<TokenStream2> = fields
        .iter()
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let field_str = field_name.to_string();
            let ty = &f.ty;
            let opts = parse_pydantic_field_opts(&f.attrs);

            match opts.serialize_with.as_deref() {
                Some("to_string") => {
                    if option_inner(ty).is_some() {
                        quote! {
                            dict.set_item(#field_str, self.#field_name.as_ref().map(|v| v.to_string()))?;
                        }
                    } else {
                        quote! {
                            dict.set_item(#field_str, self.#field_name.to_string())?;
                        }
                    }
                }
                Some("nested") => nested_serialize_stmt(field_name, &field_str, ty),
                _ => quote! {
                    dict.set_item(#field_str, self.#field_name.clone())?;
                },
            }
        })
        .collect();

    let schema = schema_method();
    let model_validate = model_validate_method();
    let model_dump = model_dump_struct_method();

    quote! {
        #[cfg(feature = "python")]
        #[::pyo3::pymethods]
        impl #name {
            #schema

            #[classmethod]
            fn _pydantic_validate(
                _cls: &::pyo3::Bound<'_, ::pyo3::types::PyType>,
                value: &::pyo3::Bound<'_, ::pyo3::PyAny>,
            ) -> ::pyo3::PyResult<Self> {
                use ::pyo3::types::PyAnyMethods as _;
                if let Ok(v) = value.extract::<Self>() {
                    return Ok(v);
                }
                Ok(Self {
                    #( #validate_fields )*
                })
            }

            fn _pydantic_serialize<'py>(
                &self,
                py: ::pyo3::Python<'py>,
            ) -> ::pyo3::PyResult<::pyo3::Bound<'py, ::pyo3::types::PyDict>> {
                use ::pyo3::types::PyDictMethods as _;
                let dict = ::pyo3::types::PyDict::new(py);
                #( #serialize_fields )*
                Ok(dict)
            }

            #model_validate
            #model_dump
        }
    }
}

// ── enum generation ────────────────────────────────────────────────────────

fn generate_enum_impl(name: &syn::Ident, type_opts: &TypeOpts) -> TokenStream2 {
    let schema = schema_method();

    let validate_body = match type_opts.validate.as_deref() {
        Some("from_str") => {
            let ty_ts = quote! { Self };
            let obj = quote! { value };
            let extraction = extract_or_from_str(&ty_ts, &obj);
            quote! { Ok(#extraction) }
        }
        _ => quote! {
            value.extract::<Self>().map_err(|e| e)
        },
    };

    let serialize_body: TokenStream2 = match type_opts.serialize.as_deref() {
        Some("to_string") => quote! { self.to_string() },
        _ => quote! { compile_error!(
            "PydanticCompat on an enum requires #[pydantic(serialize = \"to_string\")]"
        ) },
    };

    let model_validate = model_validate_method();
    let model_dump = model_dump_enum_method();

    quote! {
        #[cfg(feature = "python")]
        #[::pyo3::pymethods]
        impl #name {
            #schema

            #[classmethod]
            fn _pydantic_validate(
                _cls: &::pyo3::Bound<'_, ::pyo3::types::PyType>,
                value: &::pyo3::Bound<'_, ::pyo3::PyAny>,
            ) -> ::pyo3::PyResult<Self> {
                use ::pyo3::types::PyAnyMethods as _;
                #validate_body
            }

            fn _pydantic_serialize(&self) -> String {
                #serialize_body
            }

            #model_validate
            #model_dump
        }
    }
}

// ── entry point ────────────────────────────────────────────────────────────

/// Derive pydantic v2 compatibility for a PyO3 `#[pyclass]`.
///
/// Generates a `#[pymethods]` impl block (in a `#[cfg(feature = "python")]` guard)
/// with five methods:
///
/// - **`__get_pydantic_core_schema__`** — registers a plain validator + serializer
///   with pydantic-core so the type works natively in pydantic models.
/// - **`_pydantic_validate`** — low-level: accepts an existing instance (pass-through)
///   or a Python dict/str.
/// - **`_pydantic_serialize`** — low-level: converts to `dict` (structs) or `str` (enums).
/// - **`model_validate(cls, obj)`** — high-level API mirroring `BaseModel.model_validate`.
/// - **`model_dump(*, mode=None)`** — high-level API mirroring `BaseModel.model_dump`.
///
/// # Field attributes (structs)
///
/// ```rust,ignore
/// #[pydantic(validate_with = "from_str")]   // try extract first, then parse from str
/// #[pydantic(validate_with = "nested")]     // try extract first, then call T::_pydantic_validate
///                                           // auto-handles Option<T> and Vec<T>
/// #[pydantic(serialize_with = "to_string")] // serialise with Display::to_string
///                                           // auto-handles Option<T>
/// #[pydantic(serialize_with = "nested")]    // call ._pydantic_serialize() on the value
///                                           // auto-handles Option<T> and Vec<T>
/// ```
///
/// # Type attributes (enums)
///
/// ```rust,ignore
/// #[pydantic(validate = "from_str")]   // try extract first, then parse from str
/// #[pydantic(serialize = "to_string")] // required for enums
/// ```
#[proc_macro_derive(PydanticCompat, attributes(pydantic))]
pub fn derive_pydantic_compat(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let type_opts = parse_pydantic_type_opts(&input.attrs);

    let expanded = match &input.data {
        Data::Struct(data) => generate_struct_impl(name, data),
        Data::Enum(_) => generate_enum_impl(name, &type_opts),
        Data::Union(_) => panic!("PydanticCompat does not support unions"),
    };

    TokenStream::from(expanded)
}
