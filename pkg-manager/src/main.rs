//! hairr OS Package Manager
//! 
//! Native package management system for installing, updating, and managing
//! applications and system components on hairr OS.

use std::collections::HashMap;
use std::io::{self, Write};

/// Package identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageId(String);

impl PackageId {
    pub fn new(id: String) -> Self {
        PackageId(id)
    }
}

impl From<String> for PackageId {
    fn from(id: String) -> Self {
        PackageId(id)
    }
}

impl From<&str> for PackageId {
    fn from(id: &str) -> Self {
        PackageId(id.to_string())
    }
}

/// Package version
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Version { major, minor, patch }
    }

    pub fn parse(version_str: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid version format".to_string());
        }

        let major = parts[0].parse().map_err(|_| "Invalid major version")?;
        let minor = parts[1].parse().map_err(|_| "Invalid minor version")?;
        let patch = parts[2].parse().map_err(|_| "Invalid patch version")?;

        Ok(Version { major, minor, patch })
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Package metadata
#[derive(Debug, Clone)]
pub struct Package {
    pub id: PackageId,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub author: String,
    pub dependencies: Vec<PackageId>,
    pub installed: bool,
    pub size: u64,
}

impl Package {
    pub fn new(id: PackageId, name: String, version: Version, description: String) -> Self {
        Package {
            id,
            name,
            version,
            description,
            author: String::new(),
            dependencies: Vec::new(),
            installed: false,
            size: 0,
        }
    }
}

/// Package repository
pub struct Repository {
    url: String,
    packages: HashMap<PackageId, Package>,
}

impl Repository {
    pub fn new(url: String) -> Self {
        Repository {
            url,
            packages: HashMap::new(),
        }
    }

    pub fn add_package(&mut self, package: Package) {
        self.packages.insert(package.id.clone(), package);
    }

    pub fn find_package(&self, id: &PackageId) -> Option<&Package> {
        self.packages.get(id)
    }

    pub fn search(&self, query: &str) -> Vec<&Package> {
        self.packages
            .values()
            .filter(|p| p.name.contains(query) || p.description.contains(query))
            .collect()
    }
}

/// Package manager
pub struct PackageManager {
    repositories: Vec<Repository>,
    installed_packages: HashMap<PackageId, Package>,
}

impl PackageManager {
    pub fn new() -> Self {
        let mut manager = PackageManager {
            repositories: Vec::new(),
            installed_packages: HashMap::new(),
        };

        // Initialize with default repository
        let mut default_repo = Repository::new("https://packages.hairr-os.org".to_string());
        
        // Add some default packages
        default_repo.add_package(Package::new(
            PackageId::from("text-editor"),
            "Text Editor".to_string(),
            Version::new(1, 0, 0),
            "A simple text editor for hairr OS".to_string(),
        ));
        
        default_repo.add_package(Package::new(
            PackageId::from("file-manager"),
            "File Manager".to_string(),
            Version::new(1, 2, 0),
            "Browse and manage files".to_string(),
        ));
        
        default_repo.add_package(Package::new(
            PackageId::from("web-browser"),
            "Web Browser".to_string(),
            Version::new(2, 1, 5),
            "Modern web browser".to_string(),
        ));
        
        default_repo.add_package(Package::new(
            PackageId::from("chrysalis"),
            "Chrysalis Compatibility Suite".to_string(),
            Version::new(0, 9, 0),
            "Run Linux and Android applications".to_string(),
        ));

        manager.repositories.push(default_repo);
        manager
    }

    /// Install a package
    pub fn install(&mut self, package_id: &PackageId) -> Result<(), String> {
        // Check if already installed
        if self.installed_packages.contains_key(package_id) {
            return Err("Package already installed".to_string());
        }

        // Find package in repositories
        let package = self
            .find_package_in_repos(package_id)
            .ok_or("Package not found in any repository")?
            .clone();

        // Install dependencies first
        for dep_id in &package.dependencies {
            if !self.installed_packages.contains_key(dep_id) {
                self.install(dep_id)?;
            }
        }

        // Install the package
        let mut installed_package = package;
        installed_package.installed = true;
        self.installed_packages.insert(package_id.clone(), installed_package);

        Ok(())
    }

    /// Uninstall a package
    pub fn uninstall(&mut self, package_id: &PackageId) -> Result<(), String> {
        if !self.installed_packages.contains_key(package_id) {
            return Err("Package not installed".to_string());
        }

        // Check for dependent packages
        let dependents = self.find_dependents(package_id);
        if !dependents.is_empty() {
            return Err(format!(
                "Cannot uninstall: required by {:?}",
                dependents
            ));
        }

        self.installed_packages.remove(package_id);
        Ok(())
    }

    /// Update a package
    pub fn update(&mut self, package_id: &PackageId) -> Result<(), String> {
        if !self.installed_packages.contains_key(package_id) {
            return Err("Package not installed".to_string());
        }

        // Find latest version in repositories
        let latest = self
            .find_package_in_repos(package_id)
            .ok_or("Package not found in any repository")?
            .clone();

        let mut updated_package = latest;
        updated_package.installed = true;
        self.installed_packages.insert(package_id.clone(), updated_package);

        Ok(())
    }

    /// List installed packages
    pub fn list_installed(&self) -> Vec<&Package> {
        self.installed_packages.values().collect()
    }

    /// Search for packages
    pub fn search(&self, query: &str) -> Vec<Package> {
        let mut results = Vec::new();
        for repo in &self.repositories {
            results.extend(repo.search(query).into_iter().cloned());
        }
        results
    }

    /// Get package information
    pub fn info(&self, package_id: &PackageId) -> Option<Package> {
        self.installed_packages
            .get(package_id)
            .cloned()
            .or_else(|| self.find_package_in_repos(package_id).cloned())
    }

    fn find_package_in_repos(&self, package_id: &PackageId) -> Option<&Package> {
        for repo in &self.repositories {
            if let Some(package) = repo.find_package(package_id) {
                return Some(package);
            }
        }
        None
    }

    fn find_dependents(&self, package_id: &PackageId) -> Vec<PackageId> {
        self.installed_packages
            .values()
            .filter(|p| p.dependencies.contains(package_id))
            .map(|p| p.id.clone())
            .collect()
    }
}

impl Default for PackageManager {
    fn default() -> Self {
        Self::new()
    }
}

/// CLI for package manager
pub struct CLI {
    manager: PackageManager,
}

impl CLI {
    pub fn new() -> Self {
        CLI {
            manager: PackageManager::new(),
        }
    }

    pub fn run(&mut self) {
        println!("hairr Package Manager v0.1.0");
        println!("Type 'help' for available commands\n");

        loop {
            print!("pkg> ");
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
            "install" => {
                if parts.len() < 2 {
                    println!("Usage: install <package_id>");
                } else {
                    let package_id = PackageId::from(parts[1]);
                    match self.manager.install(&package_id) {
                        Ok(_) => println!("Package installed successfully"),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                Ok(false)
            }
            "uninstall" | "remove" => {
                if parts.len() < 2 {
                    println!("Usage: uninstall <package_id>");
                } else {
                    let package_id = PackageId::from(parts[1]);
                    match self.manager.uninstall(&package_id) {
                        Ok(_) => println!("Package uninstalled successfully"),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                Ok(false)
            }
            "update" => {
                if parts.len() < 2 {
                    println!("Usage: update <package_id>");
                } else {
                    let package_id = PackageId::from(parts[1]);
                    match self.manager.update(&package_id) {
                        Ok(_) => println!("Package updated successfully"),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                Ok(false)
            }
            "list" => {
                self.list_packages();
                Ok(false)
            }
            "search" => {
                if parts.len() < 2 {
                    println!("Usage: search <query>");
                } else {
                    let query = parts[1..].join(" ");
                    self.search_packages(&query);
                }
                Ok(false)
            }
            "info" => {
                if parts.len() < 2 {
                    println!("Usage: info <package_id>");
                } else {
                    let package_id = PackageId::from(parts[1]);
                    self.show_info(&package_id);
                }
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
        println!("  install <package>    - Install a package");
        println!("  uninstall <package>  - Uninstall a package");
        println!("  update <package>     - Update a package");
        println!("  list                 - List installed packages");
        println!("  search <query>       - Search for packages");
        println!("  info <package>       - Show package information");
        println!("  help                 - Show this help message");
        println!("  exit/quit            - Exit the package manager");
    }

    fn list_packages(&self) {
        let packages = self.manager.list_installed();
        if packages.is_empty() {
            println!("No packages installed");
            return;
        }

        println!("\nInstalled Packages:");
        println!("{:<20} {:<10} {:<50}", "Name", "Version", "Description");
        println!("{:-<80}", "");
        
        for package in packages {
            println!(
                "{:<20} {:<10} {:<50}",
                package.name,
                package.version,
                package.description
            );
        }
        println!();
    }

    fn search_packages(&self, query: &str) {
        let results = self.manager.search(query);
        if results.is_empty() {
            println!("No packages found matching '{}'", query);
            return;
        }

        println!("\nSearch Results for '{}':", query);
        println!("{:<20} {:<10} {:<50}", "Name", "Version", "Description");
        println!("{:-<80}", "");
        
        for package in results {
            let installed = if package.installed { "[installed]" } else { "" };
            println!(
                "{:<20} {:<10} {:<40} {}",
                package.name,
                package.version,
                package.description,
                installed
            );
        }
        println!();
    }

    fn show_info(&self, package_id: &PackageId) {
        if let Some(package) = self.manager.info(package_id) {
            println!("\nPackage Information:");
            println!("  Name:        {}", package.name);
            println!("  Version:     {}", package.version);
            println!("  Description: {}", package.description);
            println!("  Installed:   {}", package.installed);
            if !package.dependencies.is_empty() {
                println!("  Dependencies: {:?}", package.dependencies);
            }
            println!();
        } else {
            println!("Package not found");
        }
    }
}

fn main() {
    let mut cli = CLI::new();
    cli.run();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        let version = Version::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn test_package_installation() {
        let mut manager = PackageManager::new();
        let package_id = PackageId::from("text-editor");
        
        assert!(manager.install(&package_id).is_ok());
        assert!(manager.installed_packages.contains_key(&package_id));
    }

    #[test]
    fn test_package_search() {
        let manager = PackageManager::new();
        let results = manager.search("browser");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_duplicate_installation() {
        let mut manager = PackageManager::new();
        let package_id = PackageId::from("text-editor");
        
        assert!(manager.install(&package_id).is_ok());
        assert!(manager.install(&package_id).is_err());
    }
}
