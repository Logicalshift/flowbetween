# FlowBetween

FlowBetween is a SVG-based animation tool.

## Technical stuff

This repo contains some neat technolgy that was used to build FlowBetween and
may be useful in other projects. Here are some highlights:

 * The UI library provides a data-only description of an application's UI.
 * The binding library provides a data-driven approach to event handling.
 * The curves library provides a lot of useful functions for dealing with 
   bezier curves.
 * We use a 'browser as terminal' principle (HTML/JS/CSS are only used for
   display and never for application logic)
 * The desync library provides an alternative to both threads and data
   locking structures.
