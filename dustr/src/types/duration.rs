use crate::helpers::*;
use ::syn::*;

/// The std lib's `bool` type behavior.
pub struct Behavior;

impl super::Behavior for Behavior {
    fn is(&self, sty: &Type) -> bool {
        if let Type::Path(tp) = sty {
            is_same_id(&tp.path, "Duration")
        } else {
            false
        }
    }
    fn imports(&self, _sty: &Type, _pkg: &str, _crate_name: &str) -> Vec<String> { vec![] }
    fn name(&self, _sty: &Type) -> String { "duration".to_owned() }

    fn shim(&self, _sty: &Type) -> String {
        "Int64".to_owned()
    }
    fn ffi(&self, _sty: &Type) -> String {
        "int".to_owned()
    }
    fn native(&self, _sty: &Type) -> String {
        "Duration".to_owned()
    }

    fn native_to_ffi(&self, _sty: &Type, expr: String) -> String {
        format!("{}.inMilliseconds", expr)
    }
    fn ffi_to_native(&self, _sty: &Type, expr: String) -> String {
        format!("Duration(milliseconds: {})", expr)
    }
}
