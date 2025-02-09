use std::error::Error;

pub mod server_node;

/// From active model
/// 
/// 
pub trait FromActiveModel<T, U> {
	/// Convert from active model
	/// 
	/// 
	fn from_active_model(active_models: T) -> Result<U, Box<dyn Error>>;
}

pub trait FromModel<T, U> {
	/// Convert from model
	/// 
	/// 
	fn from_model(model: T) -> Result<U, Box<dyn Error>>;
}
