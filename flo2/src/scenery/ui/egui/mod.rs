// FlowBetween, a tool for creating vector animations
// Copyright (C) 2026 Andrew Hunter
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//!
//! This provides an implementation of Flowbetween's dialog messages implemented using egui.
//!
//! The 'subprograms that send messages' design of flo_scene is in some respects quite similar to how imguis work,
//! and egui returns render requests rather than being the interface with the OS, which also suits FlowBetween's
//! design. However, event handling is done by sending messages in FlowBetween, which is quite different from how
//! imguis traditionally work.
//!

mod dialog_egui;
mod key;
mod events;
mod draw;
mod hub;
mod state;

pub use hub::*;
