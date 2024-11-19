pub enum ExitCode {
    // Lexer
    SourceFileNotFound = 1,
    FileExtentionNotTesc = 2,
    SourcePermissionDenied = 3,

    // Process
    ProcessNotFound = 21,
    ProcessPermissionDenied = 22,

    Unknown = 101,
}
