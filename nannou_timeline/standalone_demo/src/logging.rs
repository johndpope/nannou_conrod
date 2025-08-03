//! Debug console and logging functionality

#[derive(Clone)]
pub struct LogMessage {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum LogLevel {
    Info,
    Action,
    Warning,
    Error,
}

impl LogLevel {
    pub fn get_color(&self) -> egui::Color32 {
        match self {
            LogLevel::Info => egui::Color32::from_gray(200),
            LogLevel::Action => egui::Color32::from_rgb(100, 200, 255),
            LogLevel::Warning => egui::Color32::from_rgb(255, 200, 100),
            LogLevel::Error => egui::Color32::from_rgb(255, 100, 100),
        }
    }
    
    pub fn get_icon(&self) -> &'static str {
        match self {
            LogLevel::Info => "ℹ️",
            LogLevel::Action => "▶",
            LogLevel::Warning => "⚠️",
            LogLevel::Error => "❌",
        }
    }
}

pub struct Logger {
    pub messages: Vec<LogMessage>,
    pub max_messages: usize,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            max_messages: 1000,
        }
    }
    
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        self.messages.push(LogMessage {
            timestamp,
            level,
            message: message.into(),
        });
        
        // Keep only last N messages
        if self.messages.len() > self.max_messages {
            self.messages.drain(0..100);
        }
    }
    
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}