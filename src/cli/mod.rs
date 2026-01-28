use anyhow::Result;
use clap::{Parser, ValueEnum};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "nimble")]
#[command(about = "Nimble project generator", long_about = None)]
pub struct Cli {
    pub name: String,

    #[arg(short, long, value_enum)]
    pub frontend: Option<Frontend>,

    #[arg(short, long, value_enum, default_value_t = CssLib::Tailwind)]
    pub css: CssLib,

    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Frontend {
    Angular,
    React,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum CssLib {
    Tailwind,
    Bootstrap,
}

pub struct Generator {
    cli: Cli,
    output_dir: PathBuf,
}

impl Generator {
    pub fn new(cli: Cli) -> Self {
        let output_dir = cli
            .output
            .clone()
            .unwrap_or_else(|| PathBuf::from(&cli.name));
        Self { cli, output_dir }
    }

    pub fn generate(&self) -> Result<()> {
        println!("🚀 Generating project {}...", self.cli.name);

        fs::create_dir_all(&self.output_dir)?;

        self.generate_backend()?;
        self.generate_gitignore()?;
        self.generate_docker()?;

        if let Some(frontend) = self.cli.frontend {
            self.generate_frontend(frontend)?;
        }

        println!("\n✅ Project generated at {:?}", self.output_dir);
        println!("Run 'cd {:?}' to get started!", self.output_dir);
        Ok(())
    }

    fn generate_backend(&self) -> Result<()> {
        let src_dir = self.output_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        let cargo_toml = include_str!("../../templates/cli/Cargo.toml.template");
        let main_rs = include_str!("../../templates/cli/main.rs.template");

        let cargo_toml = cargo_toml.replace("{{PROJECT_NAME}}", &self.cli.name);
        let main_rs = main_rs.replace("{{PROJECT_NAME}}", &self.cli.name);

        fs::write(self.output_dir.join("Cargo.toml"), cargo_toml)?;
        fs::write(src_dir.join("main.rs"), main_rs)?;

        Ok(())
    }

    fn generate_gitignore(&self) -> Result<()> {
        let gitignore = include_str!("../../templates/cli/gitignore.template");
        fs::write(self.output_dir.join(".gitignore"), gitignore)?;
        Ok(())
    }

    fn generate_docker(&self) -> Result<()> {
        let dev = include_str!("../../templates/cli/Dockerfile.dev.template");
        let deploy = include_str!("../../templates/cli/Dockerfile.deploy.template");

        let deploy = deploy.replace("{{PROJECT_NAME}}", &self.cli.name);

        fs::write(self.output_dir.join("Dockerfile.dev"), dev)?;
        fs::write(self.output_dir.join("Dockerfile"), deploy)?;
        Ok(())
    }

    fn generate_frontend(&self, frontend: Frontend) -> Result<()> {
        let frontend_dir = self.output_dir.join("frontend");
        fs::create_dir_all(&frontend_dir)?;

        match frontend {
            Frontend::Angular => self.generate_angular(&frontend_dir)?,
            Frontend::React => self.generate_react(&frontend_dir)?,
        }
        Ok(())
    }

    fn generate_angular(&self, dir: &Path) -> Result<()> {
        println!("✨ Adding Angular frontend...");
        let pkg_json = include_str!("../../templates/cli/frontend/angular/package.json.template");
        let pkg_json = pkg_json.replace("{{PROJECT_NAME}}", &self.cli.name);
        fs::write(dir.join("package.json"), pkg_json)?;

        // In a real app we'd add more files here
        Ok(())
    }

    fn generate_react(&self, dir: &Path) -> Result<()> {
        println!("✨ Adding React frontend...");
        let src_dir = dir.join("src");
        fs::create_dir_all(&src_dir)?;

        let pkg_json = include_str!("../../templates/cli/frontend/react/package.json.template");
        let pkg_json = pkg_json.replace("{{PROJECT_NAME}}", &self.cli.name);
        fs::write(dir.join("package.json"), pkg_json)?;

        let app_js = match self.cli.css {
            CssLib::Tailwind => {
                let tailwind_config =
                    include_str!("../../templates/cli/tailwind.config.js.template");
                fs::write(dir.join("tailwind.config.js"), tailwind_config)?;
                include_str!("../../templates/cli/frontend/react/App.js.tailwind.template")
            }
            CssLib::Bootstrap => {
                include_str!("../../templates/cli/frontend/react/App.js.bootstrap.template")
            }
        };

        let app_js = app_js.replace("{{PROJECT_NAME}}", &self.cli.name);
        fs::write(src_dir.join("App.js"), app_js)?;

        Ok(())
    }
}
