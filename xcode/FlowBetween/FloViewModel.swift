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

class NotifyProperty {
    weak var property: FloProperty?;
}

class NotifyList {
    var properties: [NotifyProperty] = [];
}

///
/// Provides the view model implementation methods
///
public class FloViewModel : NSObject {
    /// The properties in this viewmodel
    fileprivate var _properties: [UInt64: PropertyValue];
    
    /// What to notify when a property changes
    fileprivate var _toNotify: [UInt64: NotifyList];
    
    override init() {
        _properties = [UInt64: PropertyValue]();
        _toNotify   = [UInt64: NotifyList]();
    }
    
    @objc public func setNothing(_ propertyId: UInt64) {
        _properties[propertyId] = PropertyValue.Nothing;
    }
    
    @objc public func setBool(_ propertyId: UInt64, toValue: Bool) {
        _properties[propertyId] = PropertyValue.Bool(toValue);
    }
    
    @objc public func setInt(_ propertyId: UInt64, toValue: Int64) {
        _properties[propertyId] = PropertyValue.Int(toValue);
    }
    
    @objc public func setFloat(_ propertyId: UInt64, toValue: Float64) {
        _properties[propertyId] = PropertyValue.Float(toValue);
    }
    
    @objc public func setString(_ propertyId: UInt64, toValue: NSString) {
        _properties[propertyId] = PropertyValue.String(toValue as String);
    }
    
    @objc public func setProperty(_ propertyId: UInt64, toValue: FloProperty) {
        _properties[propertyId] = toValue.value;
    }

    ///
    /// Notifies anything that's listening that the specified property has changed
    ///
    func notifyPropertyChanged(_ propertyId: UInt64) {
        if let notifyList = _toNotify[propertyId] {
            // Filter out any properties that have been removed
            notifyList.properties = notifyList.properties.filter({ maybeProperty in maybeProperty.property != nil });
            
            // Notify any property still in the list
            for maybeProperty in notifyList.properties {
                if let property = maybeProperty.property {
                    property.notifyChange();
                }
            }
        }
    }
    
    ///
    /// Retrieves the value of the specified property
    ///
    public func valueForProperty(_ propertyId: UInt64) -> PropertyValue {
        if let value = _properties[propertyId] {
            return value;
        } else {
            return PropertyValue.Nothing;
        }
    }
    
    ///
    /// Notifies the specified FloProperty whenever the property with the specified ID is changed
    ///
    public func watchProperty(_ propertyId: UInt64, _ property: FloProperty) {
        // Create the notifyProperty object to store the property reference
        let notifyProperty      = NotifyProperty.init();
        notifyProperty.property = property;
        
        // Add to the list to notify for this ID
        if let toNotify = _toNotify[propertyId] {
            toNotify.properties.append(notifyProperty);
        } else {
            let newNotifyList = NotifyList.init();
            newNotifyList.properties.append(notifyProperty);
            _toNotify[propertyId] = newNotifyList;
        }
    }
}
