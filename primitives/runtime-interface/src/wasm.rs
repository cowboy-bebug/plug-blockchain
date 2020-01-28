// Copyright 2019-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Traits required by the runtime interface from the wasm side.

use crate::RIType;

use sp_std::cell::Cell;

/// Something that can be created from a ffi value.
///
/// # Safety
///
/// It is unsafe behavior to call `Something::into_ffi_value().get()` and take this as input for
/// `from_ffi_value`. Implementations are safe to assume that the `arg` given to `from_ffi_value`
/// is only generated by the corresponding [`host::IntoFFIValue`](crate::host::IntoFFIValue)
/// implementation.
pub trait FromFFIValue: Sized + RIType {
	/// Create `Self` from the given ffi value.
	fn from_ffi_value(arg: Self::FFIType) -> Self;
}

/// Something that can be converted into a ffi value.
pub trait IntoFFIValue: RIType {
	/// The owned rust type that is stored with the ffi value in [`WrappedFFIValue`].
	///
	/// If no owned value is required, `()` can be used as a type.
	type Owned;

	/// Convert `self` into a [`WrappedFFIValue`].
	fn into_ffi_value(&self) -> WrappedFFIValue<Self::FFIType, Self::Owned>;
}

/// Represents a wrapped ffi value.
///
/// It is either the ffi value itself or the ffi value plus some other owned value. By providing
/// support for storing another owned value besides the actual ffi value certain performance
/// optimizations can be applied. For example using the pointer to a `Vec<u8>`, while using the
/// pointer to a SCALE encoded `Vec<u8>` that is stored in this wrapper for any other `Vec<T>`.
pub enum WrappedFFIValue<T, O = ()> {
	Wrapped(T),
	WrappedAndOwned(T, O),
}

impl<T: Copy, O> WrappedFFIValue<T, O> {
	/// Returns the wrapped ffi value.
	pub fn get(&self) -> T {
		match self {
			Self::Wrapped(data) | Self::WrappedAndOwned(data, _) => *data,
		}
	}
}

impl<T, O> From<T> for WrappedFFIValue<T, O> {
	fn from(val: T) -> Self {
		WrappedFFIValue::Wrapped(val)
	}
}

impl<T, O> From<(T, O)> for WrappedFFIValue<T, O> {
	fn from(val: (T, O)) -> Self {
		WrappedFFIValue::WrappedAndOwned(val.0, val.1)
	}
}

/// The state of an exchangeable function.
#[derive(Clone, Copy)]
enum ExchangeableFunctionState {
	/// Original function is present
	Original,
	/// The function has been replaced.
	Replaced,
}

/// A function which implementation can be exchanged.
///
/// Internally this works by swapping function pointers.
pub struct ExchangeableFunction<T>(Cell<(T, ExchangeableFunctionState)>);

impl<T> ExchangeableFunction<T> {
	/// Create a new instance of `ExchangeableFunction`.
	pub const fn new(impl_: T) -> Self {
		Self(Cell::new((impl_, ExchangeableFunctionState::Original)))
	}
}

impl<T: Copy> ExchangeableFunction<T> {
	/// Replace the implementation with `new_impl`.
	///
	/// # Panics
	///
	/// Panics when trying to replace an already replaced implementation.
	///
	/// # Returns
	///
	/// Returns the original implementation wrapped in [`RestoreImplementation`].
	pub fn replace_implementation(&'static self, new_impl: T)  -> RestoreImplementation<T> {
		if let ExchangeableFunctionState::Replaced = self.0.get().1 {
			panic!("Trying to replace an already replaced implementation!")
		}

		let old = self.0.replace((new_impl, ExchangeableFunctionState::Replaced));

		RestoreImplementation(self, Some(old.0))
	}

	/// Restore the original implementation.
	fn restore_orig_implementation(&self, orig: T) {
		self.0.set((orig, ExchangeableFunctionState::Original));
	}

	/// Returns the internal function pointer.
	pub fn get(&self) -> T {
		self.0.get().0
	}
}

// Wasm does not support threads, so this is safe; qed.
unsafe impl<T> Sync for ExchangeableFunction<T> {}

/// Restores a function implementation on drop.
///
/// Stores a static reference to the function object and the original implementation.
pub struct RestoreImplementation<T: 'static + Copy>(&'static ExchangeableFunction<T>, Option<T>);

impl<T: Copy> Drop for RestoreImplementation<T> {
	fn drop(&mut self) {
		self.0.restore_orig_implementation(self.1.take().expect("Value is only taken on drop; qed"));
	}
}
