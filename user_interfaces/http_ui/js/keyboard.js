'use strict';

///   __ _         _            _                      _ 
///  / _| |___ ___| |_____ _  _| |__  ___  __ _ _ _ __| |
/// |  _| / _ \___| / / -_) || | '_ \/ _ \/ _` | '_/ _` |
/// |_| |_\___/   |_\_\___|\_, |_.__/\___/\__,_|_| \__,_|
///                        |__/                          

/* exported flo_keyboard */

let flo_keyboard = (function () {
    let on_key_down = key_event => {
        console.log('Key down:', key_event.code);
    };

    let on_key_up = key_event => {
        console.log('Key up', key_event.code);
    };

    // Add key listeners to the window
    window.addEventListener('keydown', on_key_down);
    window.addEventListener('keyup', on_key_up);

    return { };
})();
