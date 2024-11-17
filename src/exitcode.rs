pub enum ExitCode {
    // Lexer
    SourceFileNotFound = 1,

    // Process
    ProcessNotFound = 21,
    PermissionDenied = 22,

    Unknown = 101,
}
