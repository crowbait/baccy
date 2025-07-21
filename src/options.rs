pub enum Exclude {
  /// Some parent directory (for directories: *or* itself) has exactly this name
  DirName(String),
  /// A file (no directory) has exactly this name
  FileName(String),
  /// Glob-like pattern, supporting `*`, `**`, etc. - like UNIX shell
  /// This must be fully qualified (or *wildcarded) from the root source directory
  /// Examples:
  /// - main.js     : Exclude main.js in the *root* folder only
  /// - **/main.js  : Exclude every file called "main.js" in all directories
  /// - **/*.js     : Exclude every .js file everywhere
  /// - **/dir/*.js : Exclude all .js files in all directories named "dir"
  Pattern(String),
}