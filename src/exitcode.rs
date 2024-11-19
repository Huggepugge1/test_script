pub enum ExitCode {
    // Lexer
    SourceFileNotFound = 1,
    FileExtentionNotTesc = 2,

    // Process
    ProcessNotFound = 21,
    PermissionDenied = 22,

    Unknown = 101,
}
