//
//  flo_cocoa.h
//  FlowBetween
//
//  Created by Andrew Hunter on 02/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

#ifndef flo_cocoa_h
#define flo_cocoa_h

#import <Cocoa/Cocoa.h>

///
/// Data returned as part of a painting event
///
struct AppPainting {
    int32_t pointer_id;
    double position_x;
    double position_y;
    double pressure;
    double tilt_x;
    double tilt_y;
};

typedef struct AppPainting AppPainting;

///
/// Interface used to send events for a view object
///
@class FloEvents;
@interface FloEvents : NSObject

- (void) sendClick: (NSString*) name;
- (void) sendVirtualScroll: (NSString*) name left: (uint32_t) left top: (uint32_t) top width: (uint32_t) width height: (uint32_t) height;
- (void) sendPaintStartForDevice: (uint32_t) deviceId name: (NSString*) name action: (AppPainting) action;
- (void) sendPaintContinueForDevice: (uint32_t) deviceId name: (NSString*) name action: (AppPainting) action;
- (void) sendPaintFinishForDevice: (uint32_t) deviceId name: (NSString*) name action: (AppPainting) action;
- (void) sendPaintCancelForDevice: (uint32_t) deviceId name: (NSString*) name action: (AppPainting) action;
- (void) redrawCanvasWithSize: (NSSize) size viewport: (NSRect) viewport;

@end

///
/// Interface used to manage a Flo session
///
@class FloControl;
@interface FloControl : NSObject

- (void) tick;

@end

/// Creates a new FlowBetween session
extern FloControl* create_flo_session(Class window_class, Class view_class, Class view_model_class);

#endif /* flo_cocoa_h */
