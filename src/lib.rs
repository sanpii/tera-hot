use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct Template {
    template_dir: String,
    tera: Arc<RwLock<tera::Tera>>,
}

impl Template {
    #[must_use]
    pub fn new(template_dir: &str) -> Self {
        let mut tera = tera::Tera::default();

        let path = format!("{template_dir}/**/*");
        if let Err(err) = tera.load_from_glob(&path) {
            panic!("Parsing error(s): {err}");
        }

        Self {
            tera: Arc::new(RwLock::new(tera)),
            template_dir: template_dir.to_string(),
        }
    }

    pub fn render(&self, template: &str, context: &tera::Context) -> tera::TeraResult<String> {
        let tera = self.tera.read().unwrap();

        tera.render(template, context)
    }

    #[cfg(debug_assertions)]
    pub fn watch(self) {
        use notify::Watcher;

        let mut watcher = notify::recommended_watcher(move |res| match res {
            Ok(event) => {
                log::info!("reloading templates: {event:?}");

                let mut tera = self.tera.write().unwrap();

                match tera.full_reload() {
                    Ok(_) => log::info!("templates reloaded"),
                    Err(e) => log::error!("failed to reload templates: {e}"),
                }
            }
            Err(e) => log::warn!("watch error: {e:?}"),
        })
        .unwrap();

        log::debug!("watching {} for changes", self.template_dir);

        watcher
            .watch(
                &std::path::PathBuf::from(&self.template_dir),
                notify::RecursiveMode::Recursive,
            )
            .unwrap();
    }

    pub fn register_function<Func, Res>(
        &mut self,
        name: impl Into<std::borrow::Cow<'static, str>>,
        function: Func,
    ) where
        Func: Fn(tera::Kwargs, &tera::State<'_>) -> Res + Sync + Send + 'static,
        Res: tera::FunctionResult,
    {
        let mut tera = self.tera.write().unwrap();

        tera.register_function(name, function);
    }

    pub fn register_filter<Func, Arg, Res>(
        &mut self,
        name: impl Into<std::borrow::Cow<'static, str>>,
        filter: Func,
    ) where
        Func: tera::Filter<Arg, Res>
            + for<'a> tera::Filter<<Arg as tera::ArgFromValue<'a>>::Output, Res>,
        Arg: for<'a> tera::ArgFromValue<'a>,
        Res: tera::FunctionResult,
    {
        let mut tera = self.tera.write().unwrap();

        tera.register_filter(name, filter);
    }

    pub fn register_test<Func, Arg, Res>(
        &mut self,
        name: impl Into<std::borrow::Cow<'static, str>>,
        test: Func,
    ) where
        Func:
            tera::Test<Arg, Res> + for<'a> tera::Test<<Arg as tera::ArgFromValue<'a>>::Output, Res>,
        Arg: for<'a> tera::ArgFromValue<'a>,
        Res: tera::TestResult,
    {
        let mut tera = self.tera.write().unwrap();

        tera.register_test(name, test);
    }

    #[cfg(not(debug_assertions))]
    pub fn watch(self) {}
}
