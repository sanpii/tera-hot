use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Template {
    template_dir: String,
    tera: Arc<RwLock<tera::Tera>>,
}

impl Template {
    #[must_use]
    pub fn new(template_dir: &str) -> Self {
        let path = format!("{template_dir}/**/*");
        let tera = match tera::Tera::new(&path) {
            Ok(tera) => tera,
            Err(err) => panic!("Parsing error(s): {err}"),
        };

        Self {
            tera: Arc::new(RwLock::new(tera)),
            template_dir: template_dir.to_string(),
        }
    }

    pub fn render(&self, template: &str, context: &tera::Context) -> tera::Result<String> {
        let tera = self.tera.read().unwrap();

        tera.render(template, context)
    }

    #[cfg(debug_assertions)]
    pub fn watch(self) {
        use notify::Watcher;

        let mut watcher = notify::recommended_watcher(move |res| {
            match res {
                Ok(event) => {
                    log::info!("reloading templates: {event:?}");

                    let mut tera = self.tera.write().unwrap();

                    match tera.full_reload() {
                        Ok(_) => log::info!("templates reloaded"),
                        Err(e) => log::error!("failed to reload templates: {e}"),
                    }
                }
                Err(e) => log::warn!("watch error: {e:?}"),
            }
        }).unwrap();

        log::debug!("watching {} for changes", self.template_dir);

        watcher
            .watch(&std::path::PathBuf::from(&self.template_dir), notify::RecursiveMode::Recursive)
            .unwrap();
    }

    pub fn register_function<F: tera::Function + 'static>(&mut self, name: &str, function: F) {
        let mut tera = self.tera.write().unwrap();

        tera.register_function(name, function)
    }

    pub fn register_filter<F: tera::Filter + 'static>(&mut self, name: &str, filter: F) {
        let mut tera = self.tera.write().unwrap();

        tera.register_filter(name, filter)
    }

    pub fn register_tester<T: tera::Test + 'static>(&mut self, name: &str, tester: T) {
        let mut tera = self.tera.write().unwrap();

        tera.register_tester(name, tester)
    }

    #[cfg(not(debug_assertions))]
    pub fn watch(self) {}
}
