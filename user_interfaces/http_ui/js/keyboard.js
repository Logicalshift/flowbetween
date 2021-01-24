'use strict';

///   __ _         _            _                      _ 
///  / _| |___ ___| |_____ _  _| |__  ___  __ _ _ _ __| |
/// |  _| / _ \___| / / -_) || | '_ \/ _ \/ _` | '_/ _` |
/// |_| |_\___/   |_\_\___|\_, |_.__/\___/\__,_|_| \__,_|
///                        |__/                          

/* exported flo_keyboard */

let flo_keyboard = (function () {
    /// Maps keys (in 'key') to the JSON needed by a FlowBetween session (serialization of KeyPress in keypress.rs)
    let key_map = {
    };

    // Maps key codes to the JSON needed by a FlowBetween session
    let code_map = {
        'ShiftLeft':        'ModifierShift',
        'ControlLeft':      'ModifierCtrl',
        'AltLeft':          'ModifierAlt',
        'MetaLeft':         'ModifierMeta',
        'OSLeft':           'ModifierMeta',

        'ShiftRight':       'ModifierShift',
        'ControlRight':     'ModifierCtrl',
        'AltRight':         'ModifierAlt',
        'MetaRight':        'ModifierMeta',
        'OSRight':          'ModifierMeta',

        'Tab':              'KeyTab',

        'ArrowUp':          'KeyUp',
        'ArrowDown':        'KeyDown',
        'ArrowLeft':        'KeyLeft',
        'ArrowRight':       'KeyRight',

        'Backslash':        'KeyBackslash',
        'Slash':            'KeyForwardslash',
        'Backquote':        'KeyBacktick',
        'Comma':            'KeyComma',
        'Period':           'KeyFullstop',
        'Semicolon':        'KeySemicolon',
        'Quote':            'KeyQuote',
        'Minus':            'KeyMinus',
        'Equal':            'KeyEquals',

        'Escape':           'KeyEscape',
        'Insert':           'KeyInsert',
        'Home':             'KeyHome',
        'PageUp':           'KeyPgUp',
        'Delete':           'KeyDelete',
        'End':              'KeyEnd',
        'PageDown':         'KeyPgDown',
        'Backspace':        'KeyBackspace',
        'Enter':            'KeyEnter',

        'F1':               'KeyF1',
        'F2':               'KeyF2',
        'F3':               'KeyF3',
        'F4':               'KeyF4',
        'F5':               'KeyF5',
        'F6':               'KeyF6',
        'F7':               'KeyF7',
        'F8':               'KeyF8',
        'F9':               'KeyF9',
        'F10':              'KeyF10',
        'F11':              'KeyF11',
        'F12':              'KeyF12',
        'F13':              'KeyF13',
        'F14':              'KeyF14',
        'F15':              'KeyF15',
        'F16':              'KeyF16',

        'NumpadDivide':     'KeyNumpadDivide',
        'NumpadMultiply':   'KeyNumpadMultiply',
        'NumpadSubtract':   'KeyNumpadMinus',
        'NumpadAdd':        'KeyNumpadAdd',
        'NumpadEnter':      'KeyNumpadEnter',
        'NumpadDecimal':    'KeyNumpadDecimal'
    };

    // Define the alphabetic keys
    for (var letter_index=0; letter_index < 26; ++letter_index) {
        let letter                  = String.fromCharCode(65 + letter_index);
        let letter_lower            = String.fromCharCode(97 + letter_index);
        let keypress                = 'Key' + letter;

        key_map[letter]             = keypress;
        key_map[letter_lower]       = keypress;

        code_map['Key' + letter]    = keypress;
    }

    // ... And the numeric keys
    for (var number_index=0; number_index < 10; ++number_index) {
        let number = String.fromCharCode(48+number_index);

        code_map['Digit' + number]  = 'Key' + number;
        code_map['Numpad' + number] = 'KeyNumpad' + number;
    }

    // Returns the 'flo' keycode for a keyboard event (null if not set)
    let to_flo_keycode = key_event => {
        return key_map[key_event.key] || code_map[key_event.code] || null;
    }

    // The set of keys that are currently pressed down
    let down_keys = [];

    /// When the user presses a key, we add it to the list of 'down' keys and send the complete set to the session
    ///  If no modifier keys are down (except Shift), then we don't send the keypress if any controls have focus
    let on_key_down = key_event => {
        let keycode = to_flo_keycode(key_event);
        if (keycode) {
            if (!down_keys.includes(keycode)) {
                down_keys.push(keycode);

                console.log("Down_keys", down_keys);
            }
        }
    };

    /// When the user releases a key, we just remove it from the list of 'down' keys
    /// TODO: cmd+C in Chrome on OS X does not result in an event for the release of the 'C' key
    let on_key_up = key_event => {
        let keycode = to_flo_keycode(key_event);
        if (keycode) {
            let down_index = down_keys.indexOf(keycode);
            if (down_index > -1) {
                down_keys.splice(down_index, 1);

                console.log("Down_keys", down_keys);
            }
        }
    };

    // If a keypress results in the window losing focus we can miss the 'up' event (eg, 'tab' is particularly bad at this)
    let on_focus = () => {
        down_keys = [];
        console.log("Focus", down_keys);
    }

    // Add key listeners to the window
    window.addEventListener('keydown', on_key_down,);
    window.addEventListener('keyup', on_key_up);
    window.addEventListener('focus', on_focus);

    return { };
})();
