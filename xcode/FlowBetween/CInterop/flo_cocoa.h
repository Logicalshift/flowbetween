//
//  flo_cocoa.h
//  FlowBetween
//
//  Created by Andrew Hunter on 02/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

#ifndef flo_cocoa_h
#define flo_cocoa_h

//
// TODO: we're getting very odd behaviour when things are released or passed between Swift and Rust
// This currently means that sessions never end :-/
//
// I suspect the root cause is that Rust's Objective-C library does not support ARC, and Swift assumes
// that everything in its interop header (here) does. Haven't been able to find a way to declare the
// header file as non-ARC. (An additional problem is that it sometimes expects symbols defining the
// classes - in particular when Unmanaged<Foo> is used - and I don't know how to declare them
// in Rust)
//

#import <Cocoa/Cocoa.h>
#import <Metal/Metal.h>
#import <QuartzCore/QuartzCore.h>

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
- (void) sendDismiss: (NSString*) name;
- (void) sendFocus: (NSString*) name;
- (void) sendChangeValue: (NSString*) name isSet: (BOOL) isSet withBool: (BOOL) value;
- (void) sendChangeValue: (NSString*) name isSet: (BOOL) isSet withDouble: (double) value;
- (void) sendChangeValue: (NSString*) name isSet: (BOOL) isSet withString: (NSString*) value;
- (void) sendVirtualScroll: (NSString*) name left: (uint32_t) left top: (uint32_t) top width: (uint32_t) width height: (uint32_t) height;
- (void) sendDrag: (NSString*) name dragAction: (uint32_t) action fromX: (double) fromX fromY: (double) fromY toX: (double) toX toY: (double) toY;
- (void) sendPaintStartForDevice: (uint32_t) deviceId name: (NSString*) name action: (AppPainting) action;
- (void) sendPaintContinueForDevice: (uint32_t) deviceId name: (NSString*) name action: (AppPainting) action;
- (void) sendPaintFinishForDevice: (uint32_t) deviceId name: (NSString*) name action: (AppPainting) action;
- (void) sendPaintCancelForDevice: (uint32_t) deviceId name: (NSString*) name action: (AppPainting) action;
- (void) redrawCanvasWithSize: (NSSize) size viewport: (NSRect) viewport;
- (void) redrawGpuCanvasWithDrawable: (id<CAMetalDrawable>*) drawable size: (NSSize) size viewport: (NSRect) viewport resolution: (CGFloat) resolution;

@end

/// FloProperty is exported from the Swift side and created by the Rust side
@class FloProperty;

/// FloCacheLayer is defined on the Swift side and returned to the Rust side
@class FloCacheLayer;

///
/// Interface used to manage a Flo session
///
@class FloControl;
@interface FloControl : NSObject

- (void) tick;
- (uint64_t) sessionId;

@end

///
/// The protocol that the Rust side uses to send data to the FloView class
///
@protocol FloViewDelegate

- (void) requestClick: (FloEvents*) events withName: (NSString*) name;
- (void) requestDismiss: (FloEvents*) events withName: (NSString*) name;
- (void) requestVirtualScroll: (FloEvents*) events withName: (NSString*) name width: (double) width height: (double) height;
- (void) requestPaintWithDeviceId: (uint32_t) deviceId events: (FloEvents*) events withName: (NSString*) name;
- (void) requestDrag: (FloEvents*) events withName: (NSString*) name;
- (void) requestFocused: (FloEvents*) events withName: (NSString*) name;
- (void) requestEditValue: (FloEvents*) events withName: (NSString*) name;
- (void) requestSetValue: (FloEvents*) events withName: (NSString*) name;
- (void) requestCancelEdit: (FloEvents*) events withName: (NSString*) name;

- (void) viewRemoveFromSuperview;
- (void) viewAddSubView: (NSObject*) subview;
- (void) viewInsertSubView: (NSObject*) subview atIndex: (uint32_t) index;
- (void) viewSetSide: (int32_t) side at: (double) pos;
- (void) viewSetSide: (int32_t) side offset: (double) pos;
- (void) viewSetSide: (int32_t) side offset: (double) pos floating: (FloProperty*) floatingOffset;
- (void) viewSetSide: (int32_t) side stretch: (double) pos;
- (void) viewSetSideAtStart: (int32_t) side;
- (void) viewSetSideAtEnd: (int32_t) side;
- (void) viewSetSideAfter: (int32_t) side;
- (void) viewSetPaddingWithLeft: (double) left top: (double) top right: (double) right bottom: (double) bottom;
- (void) viewSetZIndex: (double) zIndex;
- (void) viewSetForegroundRed: (double) red green: (double) green blue: (double) blue alpha: (double) alpha;
- (void) viewSetBackgroundRed: (double) red green: (double) green blue: (double) blue alpha: (double) alpha;
- (void) viewSetText: (FloProperty*) text;
- (void) viewSetImage: (NSImage*) image;
- (void) viewSetFontSize: (double) size;
- (void) viewSetFontWeight: (double) weight;
- (void) viewSetTextAlignment: (uint32_t) alignment;
- (void) viewSetScrollMinimumSizeWithWidth: (double) width height: (double) height;
- (void) viewSetHorizontalScrollVisibility: (uint32_t) visibility;
- (void) viewSetVerticalScrollVisibility: (uint32_t) visibility;

- (void) viewSetSelected: (FloProperty*) property;
- (void) viewSetBadged: (FloProperty*) property;
- (void) viewSetEnabled: (FloProperty*) property;
- (void) viewSetValue: (FloProperty*) property;
- (void) viewSetRangeWithLower: (FloProperty*) lower upper: (FloProperty*) upper;
- (void) viewSetFocusPriority: (FloProperty*) property;
- (void) viewFixScrollAxis: (uint32_t) axis;
- (void) viewAddClassName: (NSString*) className;

- (void) viewSetPopupOpen: (FloProperty*) isOpen;
- (void) viewSetPopupDirection: (uint32_t) direction;
- (void) viewSetPopupSizeWithWidth: (double) width height: (double) height;
- (void) viewSetPopupOffset: (double) offset;

- (void) viewInitialiseGpuCanvas: (FloEvents*) events;
- (void) viewRequestGpuCanvasRedraw;
- (CGContextRef) viewGetCanvasForDrawing: (FloEvents*) events layer: (uint32_t) layer_id;
- (FloCacheLayer*) viewCopyLayerWithId: (uint32_t) layer_id;
- (void) viewUpdateCache: (FloCacheLayer*) layer fromLayerWithId: (uint32_t) layer_id;
- (void) viewRestoreLayerTo: (uint32_t) layer_id fromCopy: (FloCacheLayer*) copyLayer;
- (void) viewFinishedDrawing;
- (void) viewSetTransform: (CGAffineTransform) transform;
- (void) viewClearCanvas;

@end

/// Creates a new FlowBetween session
extern FloControl* create_flo_session(Class window_class, Class view_class, Class view_model_class);

#endif /* flo_cocoa_h */
