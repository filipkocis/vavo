use super::App;

/// Plugin is a way to extend the functionality of the App, usually by adding systems or resources
/// bundled together. `App::add_plugin` is used to add a plugin to the App and `self.build()` will be called
/// immediately
///
/// Only the `build` method is required to be implemented.
pub trait Plugin {
    fn build(&self, app: &mut App);
}
