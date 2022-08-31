//! Plugin module that abstract the concept of a cln plugin
//! from a plugin manager point of view.
pub struct Plugin;

impl Plugin {
    /// create a new instance of the plugin.
    fn new() -> Self {
        Plugin {}
    }

    /// configure the plugin in order to work with cln.
    async fn configure(&mut self) -> Result<(), ()> {
        todo!("not implemented yet")
    }

    /// upgrade the plugin to a new version.
    async fn upgrade(&mut self) -> Result<(), ()> {
        todo!("not implemented yet")
    }

    /// remove the plugin and clean up all the data.
    async fn remove(&mut self) -> Result<(), ()> {
        todo!("not implemented yet")
    }
}
