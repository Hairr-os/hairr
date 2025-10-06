//! hairr OS App Store
//! 
//! First-party graphical application store for discovering and managing
//! applications on hairr OS.

use std::collections::HashMap;
use std::io::{self, Write};

/// Application category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppCategory {
    Productivity,
    Development,
    Graphics,
    Entertainment,
    Utilities,
    Education,
    Communication,
    System,
}

impl AppCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppCategory::Productivity => "Productivity",
            AppCategory::Development => "Development",
            AppCategory::Graphics => "Graphics",
            AppCategory::Entertainment => "Entertainment",
            AppCategory::Utilities => "Utilities",
            AppCategory::Education => "Education",
            AppCategory::Communication => "Communication",
            AppCategory::System => "System",
        }
    }
}

/// Application rating
#[derive(Debug, Clone, Copy)]
pub struct Rating {
    pub stars: f32,
    pub count: u32,
}

impl Rating {
    pub fn new(stars: f32, count: u32) -> Self {
        Rating { stars, count }
    }
}

/// Application listing in the store
#[derive(Debug, Clone)]
pub struct AppListing {
    pub id: String,
    pub name: String,
    pub developer: String,
    pub category: AppCategory,
    pub description: String,
    pub version: String,
    pub size_mb: u32,
    pub rating: Option<Rating>,
    pub price: f32,
    pub screenshots: Vec<String>,
    pub installed: bool,
}

impl AppListing {
    pub fn new(id: String, name: String, developer: String, category: AppCategory) -> Self {
        AppListing {
            id,
            name,
            developer,
            category,
            description: String::new(),
            version: "1.0.0".to_string(),
            size_mb: 0,
            rating: None,
            price: 0.0,
            screenshots: Vec::new(),
            installed: false,
        }
    }

    pub fn is_free(&self) -> bool {
        self.price == 0.0
    }
}

/// App Store
pub struct AppStore {
    apps: HashMap<String, AppListing>,
    featured_apps: Vec<String>,
    categories: HashMap<AppCategory, Vec<String>>,
}

impl AppStore {
    pub fn new() -> Self {
        let mut store = AppStore {
            apps: HashMap::new(),
            featured_apps: Vec::new(),
            categories: HashMap::new(),
        };

        store.populate_default_apps();
        store
    }

    fn populate_default_apps(&mut self) {
        // Add productivity apps
        let mut text_editor = AppListing::new(
            "text-editor".to_string(),
            "Text Editor".to_string(),
            "hairr OS Foundation".to_string(),
            AppCategory::Productivity,
        );
        text_editor.description = "A modern, fast text editor with syntax highlighting".to_string();
        text_editor.size_mb = 15;
        text_editor.rating = Some(Rating::new(4.5, 1250));
        self.add_app(text_editor);

        let mut file_manager = AppListing::new(
            "file-manager".to_string(),
            "File Manager".to_string(),
            "hairr OS Foundation".to_string(),
            AppCategory::Utilities,
        );
        file_manager.description = "Browse and manage your files with ease".to_string();
        file_manager.size_mb = 25;
        file_manager.rating = Some(Rating::new(4.7, 2100));
        self.add_app(file_manager);

        // Add development apps
        let mut code_editor = AppListing::new(
            "code-studio".to_string(),
            "Code Studio".to_string(),
            "DevTools Inc".to_string(),
            AppCategory::Development,
        );
        code_editor.description = "Professional IDE for multiple programming languages".to_string();
        code_editor.size_mb = 150;
        code_editor.rating = Some(Rating::new(4.8, 5400));
        self.add_app(code_editor);

        // Add communication apps
        let mut messenger = AppListing::new(
            "hairr-messenger".to_string(),
            "hairr Messenger".to_string(),
            "hairr OS Foundation".to_string(),
            AppCategory::Communication,
        );
        messenger.description = "Secure, decentralized messaging with end-to-end encryption".to_string();
        messenger.size_mb = 45;
        messenger.rating = Some(Rating::new(4.6, 3200));
        self.add_app(messenger);

        // Add entertainment apps
        let mut media_player = AppListing::new(
            "media-player".to_string(),
            "Media Player".to_string(),
            "Media Solutions".to_string(),
            AppCategory::Entertainment,
        );
        media_player.description = "Play all your favorite audio and video formats".to_string();
        media_player.size_mb = 80;
        media_player.rating = Some(Rating::new(4.4, 1800));
        self.add_app(media_player);

        // Add system apps
        let mut chrysalis = AppListing::new(
            "chrysalis".to_string(),
            "Chrysalis Compatibility Suite".to_string(),
            "hairr OS Foundation".to_string(),
            AppCategory::System,
        );
        chrysalis.description = "Run Linux and Android applications on hairr OS".to_string();
        chrysalis.size_mb = 500;
        chrysalis.rating = Some(Rating::new(4.3, 950));
        self.add_app(chrysalis);

        // Set featured apps
        self.featured_apps = vec![
            "code-studio".to_string(),
            "hairr-messenger".to_string(),
            "chrysalis".to_string(),
        ];
    }

    /// Add an app to the store
    pub fn add_app(&mut self, app: AppListing) {
        let category = app.category;
        let app_id = app.id.clone();
        
        self.apps.insert(app_id.clone(), app);
        
        self.categories
            .entry(category)
            .or_insert_with(Vec::new)
            .push(app_id);
    }

    /// Get an app by ID
    pub fn get_app(&self, id: &str) -> Option<&AppListing> {
        self.apps.get(id)
    }

    /// Search for apps
    pub fn search(&self, query: &str) -> Vec<&AppListing> {
        let query_lower = query.to_lowercase();
        self.apps
            .values()
            .filter(|app| {
                app.name.to_lowercase().contains(&query_lower)
                    || app.description.to_lowercase().contains(&query_lower)
                    || app.developer.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get apps by category
    pub fn get_by_category(&self, category: AppCategory) -> Vec<&AppListing> {
        if let Some(app_ids) = self.categories.get(&category) {
            app_ids.iter().filter_map(|id| self.apps.get(id)).collect()
        } else {
            Vec::new()
        }
    }

    /// Get featured apps
    pub fn get_featured(&self) -> Vec<&AppListing> {
        self.featured_apps
            .iter()
            .filter_map(|id| self.apps.get(id))
            .collect()
    }

    /// Get all apps
    pub fn get_all(&self) -> Vec<&AppListing> {
        self.apps.values().collect()
    }

    /// Mark an app as installed
    pub fn mark_installed(&mut self, id: &str) -> Result<(), String> {
        if let Some(app) = self.apps.get_mut(id) {
            app.installed = true;
            Ok(())
        } else {
            Err("App not found".to_string())
        }
    }

    /// Mark an app as uninstalled
    pub fn mark_uninstalled(&mut self, id: &str) -> Result<(), String> {
        if let Some(app) = self.apps.get_mut(id) {
            app.installed = false;
            Ok(())
        } else {
            Err("App not found".to_string())
        }
    }
}

impl Default for AppStore {
    fn default() -> Self {
        Self::new()
    }
}

/// CLI interface for the App Store
pub struct AppStoreCLI {
    store: AppStore,
}

impl AppStoreCLI {
    pub fn new() -> Self {
        AppStoreCLI {
            store: AppStore::new(),
        }
    }

    pub fn run(&mut self) {
        println!("hairr OS App Store v0.1.0");
        println!("Discover and install applications for hairr OS");
        println!("Type 'help' for available commands\n");

        loop {
            print!("store> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let input = input.trim();
            if input.is_empty() {
                continue;
            }

            match self.handle_command(input) {
                Ok(should_exit) => {
                    if should_exit {
                        break;
                    }
                }
                Err(e) => println!("Error: {}", e),
            }
        }
    }

    fn handle_command(&mut self, input: &str) -> Result<bool, String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Ok(false);
        }

        match parts[0] {
            "help" => {
                self.show_help();
                Ok(false)
            }
            "exit" | "quit" => {
                println!("Goodbye!");
                Ok(true)
            }
            "featured" => {
                self.show_featured();
                Ok(false)
            }
            "categories" => {
                self.show_categories();
                Ok(false)
            }
            "category" => {
                if parts.len() < 2 {
                    println!("Usage: category <category_name>");
                } else {
                    self.show_category(parts[1]);
                }
                Ok(false)
            }
            "search" => {
                if parts.len() < 2 {
                    println!("Usage: search <query>");
                } else {
                    let query = parts[1..].join(" ");
                    self.search_apps(&query);
                }
                Ok(false)
            }
            "info" => {
                if parts.len() < 2 {
                    println!("Usage: info <app_id>");
                } else {
                    self.show_app_info(parts[1]);
                }
                Ok(false)
            }
            "all" => {
                self.show_all_apps();
                Ok(false)
            }
            _ => {
                println!("Unknown command: {}", parts[0]);
                println!("Type 'help' for available commands");
                Ok(false)
            }
        }
    }

    fn show_help(&self) {
        println!("Available commands:");
        println!("  featured             - Show featured apps");
        println!("  categories           - List all categories");
        println!("  category <name>      - Show apps in a category");
        println!("  search <query>       - Search for apps");
        println!("  info <app_id>        - Show detailed app information");
        println!("  all                  - List all available apps");
        println!("  help                 - Show this help message");
        println!("  exit/quit            - Exit the app store");
    }

    fn show_featured(&self) {
        let featured = self.store.get_featured();
        println!("\n🌟 Featured Apps:");
        println!("{:-<80}", "");
        
        for app in featured {
            self.print_app_summary(app);
        }
        println!();
    }

    fn show_categories(&self) {
        println!("\nAvailable Categories:");
        println!("  - Productivity");
        println!("  - Development");
        println!("  - Graphics");
        println!("  - Entertainment");
        println!("  - Utilities");
        println!("  - Education");
        println!("  - Communication");
        println!("  - System");
        println!("\nUse 'category <name>' to view apps in a category");
        println!();
    }

    fn show_category(&self, category_name: &str) {
        let category = match category_name.to_lowercase().as_str() {
            "productivity" => AppCategory::Productivity,
            "development" => AppCategory::Development,
            "graphics" => AppCategory::Graphics,
            "entertainment" => AppCategory::Entertainment,
            "utilities" => AppCategory::Utilities,
            "education" => AppCategory::Education,
            "communication" => AppCategory::Communication,
            "system" => AppCategory::System,
            _ => {
                println!("Unknown category: {}", category_name);
                return;
            }
        };

        let apps = self.store.get_by_category(category);
        println!("\n{} Apps:", category.as_str());
        println!("{:-<80}", "");
        
        for app in apps {
            self.print_app_summary(app);
        }
        println!();
    }

    fn search_apps(&self, query: &str) {
        let results = self.store.search(query);
        
        if results.is_empty() {
            println!("No apps found matching '{}'", query);
            return;
        }

        println!("\nSearch Results for '{}':", query);
        println!("{:-<80}", "");
        
        for app in results {
            self.print_app_summary(app);
        }
        println!();
    }

    fn show_app_info(&self, app_id: &str) {
        if let Some(app) = self.store.get_app(app_id) {
            println!("\n{}", "=".repeat(80));
            println!("{}", app.name);
            println!("{}", "=".repeat(80));
            println!("Developer:    {}", app.developer);
            println!("Category:     {}", app.category.as_str());
            println!("Version:      {}", app.version);
            println!("Size:         {} MB", app.size_mb);
            println!("Price:        {}", if app.is_free() { "Free".to_string() } else { format!("${:.2}", app.price) });
            
            if let Some(rating) = app.rating {
                println!("Rating:       ⭐ {:.1}/5.0 ({} reviews)", rating.stars, rating.count);
            }
            
            println!("Installed:    {}", if app.installed { "Yes" } else { "No" });
            println!("\nDescription:");
            println!("{}", app.description);
            println!("{}", "=".repeat(80));
            println!();
        } else {
            println!("App not found: {}", app_id);
        }
    }

    fn show_all_apps(&self) {
        let apps = self.store.get_all();
        println!("\nAll Available Apps ({} total):", apps.len());
        println!("{:-<80}", "");
        
        for app in apps {
            self.print_app_summary(app);
        }
        println!();
    }

    fn print_app_summary(&self, app: &AppListing) {
        let price = if app.is_free() { "Free".to_string() } else { format!("${:.2}", app.price) };
        let rating = if let Some(r) = app.rating {
            format!("⭐ {:.1}", r.stars)
        } else {
            "N/A".to_string()
        };
        let installed = if app.installed { " [INSTALLED]" } else { "" };
        
        println!(
            "  {} - {} by {} ({}) - {}{}",
            app.id, app.name, app.developer, price, rating, installed
        );
    }
}

fn main() {
    let mut cli = AppStoreCLI::new();
    cli.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_store_creation() {
        let store = AppStore::new();
        assert!(!store.apps.is_empty());
    }

    #[test]
    fn test_app_search() {
        let store = AppStore::new();
        let results = store.search("editor");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_category_filtering() {
        let store = AppStore::new();
        let dev_apps = store.get_by_category(AppCategory::Development);
        assert!(!dev_apps.is_empty());
    }

    #[test]
    fn test_featured_apps() {
        let store = AppStore::new();
        let featured = store.get_featured();
        assert!(!featured.is_empty());
    }

    #[test]
    fn test_app_installation_marking() {
        let mut store = AppStore::new();
        let app_id = "text-editor";
        
        assert!(store.mark_installed(app_id).is_ok());
        let app = store.get_app(app_id).unwrap();
        assert!(app.installed);
    }
}
