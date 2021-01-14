use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Template {
    template_dir: String,
    tera: Arc<RwLock<tera::Tera>>,
}

impl Template {
    pub fn new(template_dir: &str) -> Self {
        let path = format!("{}/**/*", template_dir);
        let tera = match tera::Tera::new(&path) {
            Ok(tera) => tera,
            Err(err) => panic!("Parsing error(s): {}", err),
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
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            use notify::Watcher;

            let timeout = std::time::Duration::from_secs(2);
            let mut watcher = notify::watcher(tx, timeout).unwrap();

            log::debug!("watching {} for changes", self.template_dir);

            watcher
                .watch(&self.template_dir, notify::RecursiveMode::Recursive)
                .unwrap();

            loop {
                if rx.try_recv().is_ok() {
                    log::info!("shutting down template watcher");
                    return;
                }

                match rx.recv_timeout(timeout) {
                    Ok(event) => {
                        log::info!("reloading templates: {:?}", event);

                        match self.full_reload() {
                            Ok(_) => log::info!("templates reloaded"),
                            Err(e) => log::error!("failed to reload templates: {}", e),
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                    Err(e) => log::warn!("watch error: {:?}", e),
                }
            }
        });
    }

    #[cfg(debug_assertions)]
    fn full_reload(&self) -> tera::Result<()> {
        let mut tera = self.tera.write().unwrap();

        tera.full_reload()
    }

    pub fn register_function<F: tera::Function + 'static>(&mut self, name: &str, function: F) {
        let mut tera = self.tera.write().unwrap();

        tera.register_function(name, function)
    }

    pub fn register_filter<F: tera::Filter + 'static>(&mut self, name: &str, filter: F) {
        let mut tera = self.tera.write().unwrap();

        tera.register_filter(name, filter)
    }

    #[cfg(not(debug_assertions))]
    pub fn watch(self) {}
}
