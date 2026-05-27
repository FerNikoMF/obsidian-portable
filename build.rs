fn main() {
        if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
            let mut res = winres::WindowsResource::new();
            // Файл icon.ico должен находиться в корне проекта
            res.set_icon("assets\\obsidian-icon.ico"); 
            res.compile().unwrap();
        }
    }