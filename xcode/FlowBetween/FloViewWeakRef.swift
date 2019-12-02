//
//  FloWeakViewRef.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 15/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation

///
/// Class that gets around Swift's inability to create an array of weak references, in this
/// case to FloViews
///
class FloViewWeakRef {
    fileprivate weak var _floView: FloView?

    required init(floView: FloView) {
        _floView = floView
    }

    var floView: FloView? { return _floView }
}
