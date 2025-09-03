pub mod javascript;
pub mod python;
pub mod rust;

use crate::profile::ProfileRegistry;

/// Register all built-in profiles to the registry
#[allow(dead_code)]
pub fn register_builtin_profiles(registry: &mut ProfileRegistry) {
    registry.register(Box::new(rust::RustProfile));
    registry.register(Box::new(python::PythonProfile));
    registry.register(Box::new(javascript::JavaScriptProfile));
    registry.register(Box::new(javascript::TypeScriptProfile));
}
