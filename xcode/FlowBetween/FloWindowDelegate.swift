//
//  FloWindow.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 02/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation
import Cocoa

import Carbon.HIToolbox

class FloWindow : NSWindow {
    fileprivate var _lastModifierFlags = NSEvent.ModifierFlags.init();
    
    ///
    /// Returns the key code string for a key down or key up event
    ///
    func keyCodeForEvent(_ keys: String, keyCode: UInt16, modifierFlags: NSEvent.ModifierFlags) -> String? {
        if keys.count == 1 {
            // Keys will respect the keyboard layout, so we use it where we can
            // (Unfortunately, it also respects the effect of 'shift', which we don't want for some keys)
            switch keys {
            case "a", "A":  return "KeyA";
            case "b", "B":  return "KeyB";
            case "c", "C":  return "KeyC";
            case "d", "D":  return "KeyD";
            case "e", "E":  return "KeyE";
            case "f", "F":  return "KeyF";
            case "g", "G":  return "KeyG";
            case "h", "H":  return "KeyH";
            case "i", "I":  return "KeyI";
            case "j", "J":  return "KeyJ";
            case "k", "K":  return "KeyK";
            case "l", "L":  return "KeyL";
            case "m", "M":  return "KeyM";
            case "n", "N":  return "KeyN";
            case "o", "O":  return "KeyO";
            case "p", "P":  return "KeyP";
            case "q", "Q":  return "KeyQ";
            case "r", "R":  return "KeyR";
            case "s", "S":  return "KeyS";
            case "t", "T":  return "KeyT";
            case "u", "U":  return "KeyU";
            case "v", "V":  return "KeyV";
            case "w", "W":  return "KeyW";
            case "x", "X":  return "KeyX";
            case "y", "Y":  return "KeyY";
            case "z", "Z":  return "KeyZ";
            
            default:        break;
            }
            
            // Keycodes originate in Carbon (we need to use these as 'charactersIgnoringModifiers' is a misnomer and doesn't ignore shift)
            switch Int(keyCode) {
            case kVK_Tab:                   return "KeyTab";
            
            case kVK_ANSI_0:                return "Key0";
            case kVK_ANSI_1:                return "Key1";
            case kVK_ANSI_2:                return "Key2";
            case kVK_ANSI_3:                return "Key3";
            case kVK_ANSI_4:                return "Key4";
            case kVK_ANSI_5:                return "Key5";
            case kVK_ANSI_6:                return "Key6";
            case kVK_ANSI_7:                return "Key7";
            case kVK_ANSI_8:                return "Key8";
            case kVK_ANSI_9:                return "Key9";
                
            case kVK_UpArrow:               return "KeyUp";
            case kVK_DownArrow:             return "KeyDown";
            case kVK_LeftArrow:             return "KeyLeft";
            case kVK_RightArrow:            return "KeyRight";
                
            case kVK_ANSI_Backslash:        return "KeyBackslash";
            case kVK_ANSI_Slash:            return "KeyForwardslash";
            case kVK_ANSI_Grave:            return "KeyBacktick";
            case kVK_ANSI_Comma:            return "KeyComma";
            case kVK_ANSI_Period:           return "KeyFullstop";
            case kVK_ANSI_Semicolon:        return "KeySemicolon";
            case kVK_ANSI_Quote:            return "KeyQuote";
            case kVK_ANSI_Minus:            return "KeyMinus";
            case kVK_ANSI_Equal:            return "KeyEquals";
                
            case kVK_Escape:                return "KeyEscape";
            /* case kVK_Insert:             return "KeyInsert"; */
            case kVK_Home:                  return "KeyHome";
            case kVK_PageUp:                return "KeyPgUp";
            case kVK_PageDown:              return "KeyPgDown";
            case kVK_ForwardDelete:         return "KeyDelete";
            case kVK_End:                   return "KeyEnd";
            case kVK_Delete:                return "KeyBackspace";
            case kVK_Return:                return "KeyEnter";
                
            case kVK_F1:                    return "KeyF1";
            case kVK_F2:                    return "KeyF2";
            case kVK_F3:                    return "KeyF3";
            case kVK_F4:                    return "KeyF4";
            case kVK_F5:                    return "KeyF5";
            case kVK_F6:                    return "KeyF6";
            case kVK_F7:                    return "KeyF7";
            case kVK_F8:                    return "KeyF8";
            case kVK_F9:                    return "KeyF9";
            case kVK_F10:                   return "KeyF10";
            case kVK_F11:                   return "KeyF11";
            case kVK_F12:                   return "KeyF12";
            case kVK_F13:                   return "KeyF13";
            case kVK_F14:                   return "KeyF14";
            case kVK_F15:                   return "KeyF15";
            case kVK_F16:                   return "KeyF16";
                
            case kVK_ANSI_Keypad0:          return "KeyNumpad0";
            case kVK_ANSI_Keypad1:          return "KeyNumpad1";
            case kVK_ANSI_Keypad2:          return "KeyNumpad2";
            case kVK_ANSI_Keypad3:          return "KeyNumpad3";
            case kVK_ANSI_Keypad4:          return "KeyNumpad4";
            case kVK_ANSI_Keypad5:          return "KeyNumpad5";
            case kVK_ANSI_Keypad6:          return "KeyNumpad6";
            case kVK_ANSI_Keypad7:          return "KeyNumpad7";
            case kVK_ANSI_Keypad8:          return "KeyNumpad8";
            case kVK_ANSI_Keypad9:          return "KeyNumpad9";
            case kVK_ANSI_KeypadDivide:     return "KeyNumpadDivide";
            case kVK_ANSI_KeypadMultiply:   return "KeyNumpadMultiply";
            case kVK_ANSI_KeypadMinus:      return "KeyNumpadMinus";
            case kVK_ANSI_KeypadPlus:       return "KeyNumpadAdd";
            case kVK_ANSI_KeypadEnter:      return "KeyNumpadEnter";
            case kVK_ANSI_KeypadDecimal:    return "KeyNumpadDecimal";

            default: break;
            }
            
            // Failed to match any key event
            return nil;
        } else {
            return nil;
        }
    }
    
    ///
    /// The user has pressed a key
    ///
    override func keyDown(with event: NSEvent) {
        if let window_delegate = self.delegate as? FloWindowDelegate {
            if let key_code = self.keyCodeForEvent(event.charactersIgnoringModifiers!, keyCode: event.keyCode, modifierFlags: event.modifierFlags) {
                window_delegate.keyDown(key_code);
            }
        }
    }
    
    ///
    /// The user has released a key
    ///
    override func keyUp(with event: NSEvent) {
        if let window_delegate = self.delegate as? FloWindowDelegate {
            if let key_code = self.keyCodeForEvent(event.charactersIgnoringModifiers!, keyCode: event.keyCode, modifierFlags: event.modifierFlags) {
                window_delegate.keyUp(key_code);
            }
        }
    }
    
    ///
    /// The modifier flags have changed
    ///
    override func flagsChanged(with event: NSEvent) {
        if let window_delegate = self.delegate as? FloWindowDelegate {
            // Fetch the old and new modifier flags
            let modifierFlags   = event.modifierFlags;
            let lastFlags       = self._lastModifierFlags;
            
            // Generate key up/key down events for the ways that the flags have changed
            if modifierFlags.contains(.shift) && !lastFlags.contains(.shift) {
                window_delegate.keyDown("ModifierShift");
            }
            if modifierFlags.contains(.control) && !lastFlags.contains(.control) {
                window_delegate.keyDown("ModifierCtrl");
            }
            if modifierFlags.contains(.option) && !lastFlags.contains(.option) {
                window_delegate.keyDown("ModifierAlt");
            }
            if modifierFlags.contains(.command) && !lastFlags.contains(.command) {
                window_delegate.keyDown("ModifierMeta");
            }

            if !modifierFlags.contains(.shift) && lastFlags.contains(.shift) {
                window_delegate.keyUp("ModifierShift");
            }
            if !modifierFlags.contains(.control) && lastFlags.contains(.control) {
                window_delegate.keyUp("ModifierCtrl");
            }
            if !modifierFlags.contains(.option) && lastFlags.contains(.option) {
                window_delegate.keyUp("ModifierAlt");
            }
            if !modifierFlags.contains(.command) && lastFlags.contains(.command) {
                window_delegate.keyUp("ModifierMeta");
            }

            // Make a note of the current settings for the modifier flags
            self._lastModifierFlags = modifierFlags;
        }
    }
}

///
/// Represents a window created by FlowBetween
///
public class FloWindowDelegate : NSObject, NSWindowDelegate {
    ///
    /// The window itself
    ///
    fileprivate var _window: FloWindow

    ///
    /// The root view, if the window has one
    ///
    fileprivate var _rootView: FloView?

    ///
    /// The session that this window is for
    ///
    fileprivate weak var _session : NSObject?

    ///
    /// The session ID for this window
    ///
    fileprivate var _sessionId: UInt64

    @objc required init(_ session: FloControl!) {
        // Create the window
        let styleMask: NSWindow.StyleMask = [.resizable, .closable, .titled]

        _window = FloWindow(
            contentRect:    NSRect(x: 100, y: 100, width: 1600, height: 960),
            styleMask:      styleMask,
            backing:        .buffered,
            defer:          true)
        _session    = session
        _sessionId  = session.sessionId()

        // ??????? Cocoa bug ???????
        //
        // Can't tie this to FlowBetween's code at all. Whenever windowWillClose is called, the window
        // is released, regardless of the strong reference stored by this class. It's then released
        // again when this class is freed due to that strong reference, causing a crash.
        //
        // If this class doesn't stop the session after windowWillClose (which means nothing other than
        // the Swift side is freeing windows) then the window is still freed up on close. This appears
        // to be down to Cocoa.
        //
        // This adds an extra retain to the window so that it's only freed once. Not sure why a similar
        // issue does not happen for the popup window. Rust side does manual reference counting but it
        // never finds out about the window directly and doesn't double-free anything else so there's
        // nowhere to add an extra retain on that side.
        //
        // ... reproduced this with a really simple test case, this is a real bug in AppKit
        let buggyRetain = Unmanaged.passUnretained(_window).retain()
        let _ = buggyRetain

        super.init()

        _window.title = "FlowBetween session"
        _window.delegate = self
    }

    ///
    /// The window will close
    ///
    @objc public func windowWillClose(_ notification: Notification) {
        // Remove the session from the main app delegate
        if let delegate = NSApp.delegate as? FloAppDelegate {
            delegate.finishSessionWithId(_sessionId)
        }

        // Tidy up the window views (in case buggyRetain fails to work)
        _window.contentView = nil
    }

    ///
    /// Opens the window
    ///
    @objc public func windowOpen() {
        _window.makeKeyAndOrderFront(self)
    }

    ///
    /// Sets the root view of this window
    ///
    @objc public func windowSetRootView(_ view: FloView!) {
        _rootView               = view
        _window.contentView     = view.view
    }

    func triggerAllBoundsChanged() {
        // Notify all the FloViews that the bounds have changed
        var remainingViews = [_window.contentView!]

        while let nextView = remainingViews.popLast() {
            // Trigger the bounds changed event on any container views
            if let containerView = nextView as? FloContainerView {
                containerView.triggerBoundsChanged()
            }

            // Process the entire view tree
            remainingViews.append(contentsOf: nextView.subviews)
        }
    }

    ///
    /// The backing properties (colour scheme, resolution) of the window was changed
    ///
    @objc public func windowDidChangeBackingProperties(_ notification: Notification) {
        triggerAllBoundsChanged()
    }

    ///
    /// Callback when a tick has occurred
    ///
    @objc func tick() {
        if let session = _session {
            // The Rust obj-c crate doesn't provide a way to generate the linker symbols necessary to call FloControl directly
            session.perform(#selector(tick))
        }
    }

    ///
    /// Request for a tick event to be generated
    ///
    @objc public func requestTick() {
        // Cocoa doesn't really have a way to request an animation frame other than by delaying. We'll use a delay indicating 120fps here
        RunLoop.main.perform(inModes: [.default, .eventTracking, .modalPanel],
            block: {
                    self.perform(#selector(self.tick),
                                 with: nil,
                                 afterDelay: TimeInterval(1.0 / 120.0),
                                 inModes: [.default, .eventTracking, .modalPanel])
                }
            )
    }
    
    ///
    /// Sends a key down event to the session
    ///
    func keyDown(_ key: String) {
        if let session = _session {
            let key = key as NSString;
            session.perform(#selector(FloControl.keyDown(_:)), with: key);
        }
    }
    
    ///
    /// Sends a key up event to the session
    ///
    func keyUp(_ key: String) {
        if let session = _session {
            let key = key as NSString;
            session.perform(#selector(FloControl.keyUp(_:)), with: key);
        }
    }
}
