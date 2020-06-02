// Copyright 2017-2020 Parity Technologies (UK) Ltd.
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

#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	/// This API provides access to a storage space that contains a list of
	/// reserved nodes.
	///
	/// By setting and reading from the list of reserved nodes, the network will only allow
	/// a controlled set of peers to join, ensuring the privacy of the network.
	///
	/// This api is used by the `client/peerset` module to set and retrieve a list of
	/// reserved nodes
	pub trait NetworkPrivacyApi {
		/// Retrieve current list of reserved nodes.
		fn reserved_nodes() -> Option<Vec<Vec<u8>>>;

		/// Set the lit of reserved nodes.
		fn set_reserved_nodes( nodes: Vec<Vec<i8>>);
	}
}
