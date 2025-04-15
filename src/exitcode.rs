pub enum ExitCode {
    // Lexer
    SourceFileNotFound = 1,
    FileExtentionNotTesc = 2,
    SourcePermissionDenied = 3,

    // Process
    ProcessNotFound = 21,
    ProcessPermissionDenied = 22,

    // Parser
    ParserError = 31,

    // Type Checker
    TypeCheckerError = 41,

    Unknown = 101,
}
