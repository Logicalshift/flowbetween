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

extern NSObject* create_flo_session(Class window_class, Class view_class, Class view_model_class);

@class FloEvents;
@interface FloEvents

- (void) sendClick: (NSString*) name;
- (void) sendVirtualScroll: (NSString*) name left: (uint32_t) left top: (uint32_t) top width: (uint32_t) width height: (uint32_t) height;

@end

#endif /* flo_cocoa_h */
