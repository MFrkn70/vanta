use std::fmt;

pub const LOG_ENABLE_WARNING    : bool = true;
pub const LOG_ENABLE_INFO       : bool = true;

#[cfg(debug_assertions)]
pub const LOG_ENABLE_DEBUG      : bool = true;
#[cfg(debug_assertions)]
pub const LOG_ENABLE_TRACE      : bool = true;

#[cfg(not(debug_assertions))]
pub const LOG_ENABLE_DEBUG      : bool = false;
#[cfg(not(debug_assertions))]
pub const LOG_ENABLE_TRACE      : bool = false;

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Fatal,
    Error,
    Warning,
    Info,
    Debug,
    Trace,
}



pub fn initialize_logging() -> bool{

    // TODO: Create log file
    true
}

pub fn shutdown_logging(){
    // TODO: cleanup loggin - dump everything to file
}

pub fn log_output (level : LogLevel, message : &str){

    println!("{} {}", level, message);

}

#[macro_export]
macro_rules! log_fatal {
    ($($arg:tt)*) => {
        crate::core::logger::log_output(crate::core::logger::LogLevel::Fatal, &format!($($arg)*));
    }
}


#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        crate::core::logger::log_output(crate::core::logger::LogLevel::Error, &format!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {
        if crate::core::logger::LOG_ENABLE_WARNING{
            crate::core::logger::log_output(crate::core::logger::LogLevel::Warning, &format!($($arg)*));
        }
    }
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        if crate::core::logger::LOG_ENABLE_INFO{
            crate::core::logger::log_output(crate::core::logger::LogLevel::Info, &format!($($arg)*));
        }
    }
}


#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        if crate::core::logger::LOG_ENABLE_DEBUG{
            crate::core::logger::log_output(crate::core::logger::LogLevel::Debug, &format!($($arg)*));
        }
    }
}

#[macro_export]
macro_rules! log_trace {
    ($($arg:tt)*) => {
        if crate::core::logger::LOG_ENABLE_TRACE{
            crate::core::logger::log_output(crate::core::logger::LogLevel::Trace, &format!($($arg)*));
        }
    }
}


impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Fatal => write!(f, "[FATAL]: "),
            LogLevel::Error => write!(f, "[ERROR]: "),
            LogLevel::Warning => write!(f, "[WARNING]: "),
            LogLevel::Info => write!(f, "[INFO]: "),
            LogLevel::Debug => write!(f, "[DEBUG]: "),
            LogLevel::Trace => write!(f, "[TRACE]: "),
            _ => write!(f,"[LOG]: "),

        }
    }
}
