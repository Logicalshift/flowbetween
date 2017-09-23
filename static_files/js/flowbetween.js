// FlowBetween
function flowbetween() {
    let utf8 = new TextEncoder('utf-8');
    
    ///
    /// =====
    ///

    ///
    /// Returns a promise that pauses for a certain time
    ///
    let pause = (seconds) => {
        return new Promise((resolve) => {
            if (seconds <= 0) {
                resolve();
            } else {
                setTimeout(() => resolve(), seconds * 1000.0);
            }
        });
    }

    ///
    /// Performs an XmlHttpRequest to a particular uri with a JSON
    /// object, returning a promise.
    ///
    let xhr = (obj, uri, method) => {
        obj     = obj       || {};
        uri     = uri       || '/flowbetween/session';
        method  = method    || 'POST';

        let encoding    = JSON.stringify(obj);
        
        return new Promise((resolve, reject) => {
            // Prepare the request
            let req         = new XMLHttpRequest();

            req.open(method, uri);
            req.setRequestHeader('Content-Type', 'application/json; charset=UTF-8');
            
            // Completing the request completes the promise
            req.addEventListener('load', function() {
                let evt = this;
                if (evt.status < 200 || evt.status > 299) {
                    // Server error
                    reject('Server returned ' + evt.status);
                } else {
                    // Successful response
                    resolve(evt);
                }
            });
            req.addEventListener('error', function() {
                let evt = this;
                console.error(evt);
                reject(evt);
            });

            // Send the request
            req.send(utf8.encode(encoding));
        });
    }

    /// Sends a POST request
    let http_post   = (obj, uri) => xhr(obj, uri, 'POST');

    /// Sets a GET request
    let http_get    = (obj, uri) => xhr(obj, uri, 'GET');

    ///
    /// Retries an operation if it fails
    ///
    let retry       = (start_op, retrying_callback) => {
        return new Promise((resolve, reject) => {
            // These are the times we wait between retrying
            let timeouts    = [ 0, 1, 2, 5, 30 ];

            // This actually runs a try
            let run_try     = (pass) => {
                return pause(timeouts[pass])
                    .then(() => start_op())
                    .catch((reason) => {
                        // Notify the callback the first time we do a retry
                        if (pass == 0 && retrying_callback) {
                            retrying_callback();
                        }

                        // Either stop retrying or try the next pass
                        let next_pass = pass + 1;
                        if (next_pass >= timeouts.length) {
                            reject(reason);
                        } else {
                            return run_try(next_pass);
                        }
                    })
                    .then(result => resolve(result));
            }
            
            // Run the first try
            run_try(0);
        });
    };

    ///
    /// =====
    ///

    ///
    /// Creates an event as part of a request
    ///
    let make_event = (kind, parameter) => {
        if (parameter === undefined) {
            return kind;
        } else {
            let res = {};
            res[kind] = parameter;
            return res;
        }
    }

    ///
    /// Creates a request for a particular session
    ///
    let make_request = (events, session_id) => {
        let res = { events: events };
        
        if (session_id) {
            res.session_id = session_id;
        }

        return res;
    }

    ///
    /// Makes the new session request
    ///
    let new_session = () => {
        let request = make_request([ make_event("NewSession") ]);

        retry(() => http_post(request), () => console.warn('New session failed - retrying'))
        .then((response) => {
            console.log(response);
        })
        .catch((err) => {
            console.error('Could not create session', err);
        });
    }

    ///
    /// =====
    ///

    // All set up, let's go
    console.log('=== F L O W B E T W E E N ===');
    new_session();
};

flowbetween();
