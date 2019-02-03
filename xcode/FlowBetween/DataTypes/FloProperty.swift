//
//  FloProperty.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 04/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Foundation

///
/// Value of a ViewModel property
///
public enum PropertyValue {
    case Nothing;
    case Bool(Bool);
    case Int(Int64);
    case Float(Float64);
    case String(String);
}


///
/// Bridging class used to pass strings from the rust to the swift side of things
///
@objc(FloProperty) public class FloProperty : NSObject {
    /// The current value of this property
    var _value: PropertyValue;
    
    /// The viewmodel if this property is attached to one
    var _viewModel: FloViewModel?;
    
    /// Action to take when this property is changed
    var _onChange: (() -> ())?;
    
    /// The ID of the binding for this property
    var _bindingId: UInt64?;
    
    override init() {
        _value = PropertyValue.Nothing;
        
        super.init();
    }
    
    @objc(initWithBool:) init(withBool: Bool) {
        _value = PropertyValue.Bool(withBool);
        
        super.init();
    }
    
    @objc(initWithInt:) init(withInt: Int64) {
        _value = PropertyValue.Int(withInt);
        
        super.init();
    }
    
    @objc(initWithFloat:) init(withFloat: Float64) {
        _value = PropertyValue.Float(withFloat);
        
        super.init();
    }
    
    @objc(initWithString:) init(withString: NSString) {
        _value = PropertyValue.String(withString as String);
        
        super.init();
    }
    
    @objc(initWithBinding:viewModel:) init(withBinding: UInt64, viewModel: FloViewModel) {
        _viewModel  = viewModel;
        _value      = viewModel.valueForProperty(withBinding);
        _bindingId  = withBinding;
        
        super.init();
        
        viewModel.watchProperty(withBinding, self);
    }

    ///
    /// Retrieves the current value of this property
    ///
    public var value: PropertyValue {
        get {
            if let viewModel = _viewModel {
                return viewModel.valueForProperty(_bindingId!);
            } else {
                return _value;
            }
        }
    }
    
    ///
    /// Calls the specified function whenever the value of this property changes.
    /// The function is also called immediately with the current value so it can
    /// be used to initialise a property.
    ///
    public func trackValue(_ newValue: @escaping (PropertyValue) -> ()) {
        if let viewModel = _viewModel {
            let lastOnChange    = _onChange;
            let bindingId       = _bindingId!;
            
            _onChange = {
                // Run the callback
                let value = viewModel.valueForProperty(bindingId);
                newValue(value);
                
                // Allow multiple things to track this property
                if let lastOnChange = lastOnChange {
                    lastOnChange();
                }
            };
            
            // Call back immediately with the first update
            newValue(value);
        } else {
            // Not bound to a viewmodel, so just call back immediately with the current value
            newValue(_value);
        }
    }
    
    ///
    /// Notifies this value of a change from a viewmodel
    ///
    public func notifyChange() {
        if let viewModel = _viewModel {
            // Fetch the value from the viewmodel
            let bindingId   = _bindingId!;
            let newValue    = viewModel.valueForProperty(bindingId);
            
            // Update the value stored in this object
            _value = newValue;
            
            // Call the callback if it has been set
            if let onChange = _onChange {
                onChange();
            }
        }
    }
}
