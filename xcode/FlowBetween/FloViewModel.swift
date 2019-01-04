//
//  FloViewModel.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 04/01/2019.
//  Copyright © 2019 Andrew Hunter.
//
// FlowBetween is distributed under the terms of the Apache public license
//
//    Copyright 2018-2019 Andrew Hunter
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

import Foundation

///
/// Provides the view model implementation methods
///
public class FloViewModel : NSObject {
    fileprivate var _properties: [UInt64: PropertyValue];
    
    override init() {
        _properties = [UInt64: PropertyValue]();
    }
    
    @objc public func setNothing(_ propertyId: UInt64) {
        _properties[propertyId] = PropertyValue.Nothing;
    }
    
    @objc public func setBool(_ propertyId: UInt64, val: Bool) {
        _properties[propertyId] = PropertyValue.Bool(val);
    }
    
    @objc public func setInt(_ propertyId: UInt64, val: Int64) {
        _properties[propertyId] = PropertyValue.Int(val);
    }
    
    @objc public func setFloat(_ propertyId: UInt64, val: Float64) {
        _properties[propertyId] = PropertyValue.Float(val);
    }
    
    @objc public func setString(_ propertyId: UInt64, val: NSString) {
        _properties[propertyId] = PropertyValue.String(val as String);
    }
    
    @objc public func setProperty(_ propertyId: UInt64, val: FloProperty) {
        _properties[propertyId] = val.value;
    }
}
